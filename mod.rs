//! Symbol- and name-storage facilities.
//!
//! This is based on [Servo's string-cache
//! library](https://github.com/servo/string-cache); at the moment it's very
//! lifetime-unsafe for `Pooled` instances.
use std;
use std::fmt;
use std::convert::AsRef;
use std::hash::{Hash, Hasher, SipHasher};
use std::collections::BTreeMap;
use std::borrow::ToOwned;

use super::memcpy;


#[macro_use]
pub mod nameable;
pub use self::nameable::Nameable;


fn hash<T: Hash>(t: &T) -> u64 {
    let mut s = SipHasher::new();
    t.hash(&mut s);
    s.finish()
}

/// Methods required of `unpacked` types.
pub trait PackFormat {
    /// Pack an unpacked symbol in the implementing format into a generic
    /// Symbol object.
    fn pack(&self) -> Symbol;

    /// Unpack a generic Symbol into the implementing format.  This function is
    /// marked as `unsafe` because the caller must verify that the symbol is
    /// indeed packed by the receiver's implementation.
    unsafe fn unpack(sym: &Symbol) -> Self;

    /// Fetch a `&str` slice from a symbol packed in the implementing format.
    /// This function is marked as `unsafe` because the caller must verify that
    /// the symbol is indeed packed by the receiver's implementation.
    unsafe fn as_slice_from<'t>(sym: &'t Symbol) -> &'t str;
}


/* ****************************************************************
 * Pool
 */
/// A collection of symbols.
#[derive(Debug)]
pub struct Pool {
    map:  BTreeMap<u64,String>
}

impl Pool {
    /// Create a new symbol pool.
    pub fn new() -> Pool {
        Pool{map: BTreeMap::new()}
    }

    /// Fetch a symbol corresponding to the given string.
    pub fn symbol(&mut self, name: &str) -> Symbol {
        if name.len() <= INLINE_SYMBOL_MAX_LEN
        { Inline::new(name).pack() }
        else { let hsh = hash::<_, SipHasher>(&name) << 1;
               if ! self.map.contains_key(&hsh) {
                   self.map.insert(hsh, name.to_owned());
               }

               Pooled::new(hsh, self).pack()
        }
    }
}


/* ****************************************************************
 * Symbol
 */
/// An atomic `Copy`able string.  Symbol either encodes a short string
/// directly, or stores it in an external Pool.
#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub struct Symbol {
    value: [u64; 2]
}

impl Symbol {
    /// Convert the symbol to its unpacked representation.
    #[inline(always)]
    pub fn unpack(&self) -> Unpacked
    { unsafe { <Unpacked as PackFormat>::unpack(self) } }

    /// Get symbol's pack format.
    #[inline(always)]
    pub fn type_of(&self) -> Type {
        match (self.value[0] & 0x01) as u8 {
            INLINE => Type::INLINE,
            POOLED => Type::POOLED,
            _ => unreachable!()
        }
    }

}

impl AsRef<str> for Symbol {
    fn as_ref<'t>(&'t self) -> &'t str { unsafe { <Unpacked as PackFormat>::as_slice_from(self) } }
}

impl Nameable for Symbol {
    fn name(&self) -> Symbol { *self }
}

impl Hash for Symbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state);
    }
}


impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}


/* ****************************************************************
 * Inline
 */
const INLINE_SYMBOL_MAX_LEN: usize = 15;//std::mem::size_of::<Symbol>() - 1;

/// Data format for symbols packed entirely into a `Symbol` instance.
#[derive(Debug)]
pub struct Inline{len: u8, data: [u8; INLINE_SYMBOL_MAX_LEN]}

impl Inline {
    /// Create a new Inline-format symbol using the given string.
    pub fn new(name: &str) -> Self {
        panic_unless!(name.len() <= INLINE_SYMBOL_MAX_LEN,
                      "Argument to Inline::new (length {}) exceeds maximum length ({}) for an inlined symbol",
                      name.len(), INLINE_SYMBOL_MAX_LEN);
        let mut buf: [u8; INLINE_SYMBOL_MAX_LEN] = [0; INLINE_SYMBOL_MAX_LEN];

        memcpy(&mut buf[..], name.as_bytes());
        Inline{len: name.len() as u8, data: buf}
    }
}

impl AsRef<str> for Inline {
    fn as_ref<'t>(&'t self) -> &'t str {
        let src: &[u8] = self.data.as_ref();
        std::str::from_utf8(&src[..(self.len as usize)]).unwrap()
    }
}

impl PackFormat for Inline {
    // Convert the Inline symbol into a generic packed representation.
    fn pack(&self) -> Symbol {
        unsafe {
            let mut out = Symbol{ value: [0; 2] };
            let mut dest: &mut [u8; 16] = std::mem::transmute(&mut out.value);
            dest[0] = (self.len << 1) | INLINE;
            memcpy(&mut dest[1..], &self.data);
            out
        }
    }
    unsafe fn unpack(sym: &Symbol) -> Self {
        let mut out = Inline{len: 0, data: [0; INLINE_SYMBOL_MAX_LEN]};
        let src: &[u8;16] =
            std::mem::transmute(&sym.value);

        panic_unless!(src[0] & TAG_MASK == INLINE, "Invalid tag bit for inlined symbol!");
        out.len = src[0] >> 1;
        panic_unless!(out.len <= INLINE_SYMBOL_MAX_LEN as u8,
                      "Symbol length ({}) exceeds maximum ({}) for inlined symbol",
                      out.len, INLINE_SYMBOL_MAX_LEN);
        memcpy(&mut out.data[..], &src[1..(out.len as usize + 1)]);

        out
    }

    unsafe fn as_slice_from<'t>(sym: &'t Symbol) -> &'t str {
        let src: &[u8; 16] = std::mem::transmute(&sym.value);
        panic_unless!(src[0] & TAG_MASK == INLINE, "Invalid tag bit for inlined symbol!");
        let len: usize = (src[0] >> 1) as usize;
        panic_unless!(len <= INLINE_SYMBOL_MAX_LEN,
                      "Symbol length ({}) exceeds maximum ({}) for inlined symbol",
                      len, INLINE_SYMBOL_MAX_LEN);

        std::str::from_utf8(&src[1..(len + 1)]).unwrap()
    }
}

impl std::cmp::Eq for Inline {}
impl std::cmp::PartialEq for Inline {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.data == other.data
    }
}


/* ****************************************************************
 * Pooled
 */
/// Data format for symbols stored as references to a symbol pool.
#[derive(Debug)]
pub struct Pooled{key: u64, pool: *const Pool}

impl Pooled {
    #[inline(always)]
    pub fn new(key: u64, pool: *const Pool) -> Self {
        Pooled{key: key, pool: pool}
    }

}

impl AsRef<str> for Pooled {
    fn as_ref<'u>(&'u self) -> &'u str {
        unsafe { (*self.pool).map[&self.key].as_ref() }
    }
}

impl PackFormat for Pooled {
    fn pack(&self) -> Symbol {
        let mut val: [u64; 2] = [0; 2];
        val[0] = self.key  | POOLED as u64;
        val[1] = self.pool as u64;
        Symbol{value: val}
    }
    unsafe fn unpack(sym: &Symbol) -> Self {
        Pooled{key: sym.value[0],
               pool: std::mem::transmute(sym.value[1] as *const Pool)}
    }

    unsafe fn as_slice_from<'t>(sym: &'t Symbol) -> &'t str {
        panic_unless!(sym.value[0] & TAG_MASK as u64 == POOLED as u64,
                      "Invalid flag bit for pooled symbol");
        std::mem::transmute::<_,&'t Pool>(sym.value[1] as *const Pool).
            map[&sym.value[0]].as_ref()
    }
}


impl std::cmp::Eq for Pooled {}
impl std::cmp::PartialEq for Pooled {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.pool as u64 == other.pool as u64
    }
}


/* ****************************************************************
 * Types
 */
const TAG_MASK: u8 = 0x01;
const POOLED: u8 = 0;
const INLINE: u8 = 1;

pub enum Type { INLINE = 0, POOLED = 1 }

impl std::cmp::Eq for Type {}
impl std::cmp::PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match *self { Type::INLINE => match *other { Type::INLINE => true, _ => false },
                      Type::POOLED => match *other { Type::POOLED => true, _ => false } }
    }
}

impl std::fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self { Type::INLINE => write!(f, "inline"),
                      Type::POOLED => write!(f, "pooled") }
    }
}


/* ****************************************************************
 * Unpacked
 */

/// A differentiated `Symbol`.
#[derive(Debug)]
pub enum Unpacked {
    /// Symbol stored inline in `Symbol`'s data fields.
    Inline(Inline),

    /// Symbol stored in a Pool
    Pooled(Pooled)
}

impl PackFormat for Unpacked {
    fn pack(&self) -> Symbol {
        match *self {
            Unpacked::Inline(ref x) => x.pack(),
            Unpacked::Pooled(ref x) => x.pack()
        }
    }

    unsafe fn unpack(sym: &Symbol) -> Unpacked {
        match sym.type_of() {
            Type::POOLED => Unpacked::Pooled(<Pooled as PackFormat>::unpack(sym)),
            Type::INLINE => Unpacked::Inline(<Inline as PackFormat>::unpack(sym)),
        }
    }

    unsafe fn as_slice_from<'t>(sym: &'t Symbol) -> &'t str {
        match sym.type_of() {
            Type::INLINE => <Inline as PackFormat>::as_slice_from(sym),
            Type::POOLED => <Pooled as PackFormat>::as_slice_from(sym)
        }
    }
}

impl AsRef<str> for Unpacked {
    fn as_ref<'t>(&'t self) -> &'t str {
        match *self {
            Unpacked::Inline(ref x) => x.as_ref(),
            Unpacked::Pooled(ref x) => x.as_ref()
        }
    }
}

impl std::cmp::Eq for Unpacked {}
impl std::cmp::PartialEq for Unpacked {
    fn eq(&self, other: &Self) -> bool {
        match *self {
            Unpacked::Inline(ref si) =>
                match *other { Unpacked::Inline(ref oi) => si == oi, _ => false },
            Unpacked::Pooled(ref sp) =>
                match *other { Unpacked::Pooled(ref op) => sp == op, _ => false }
        }
    }
}


impl Unpacked {
}
