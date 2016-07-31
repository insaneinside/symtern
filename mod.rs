//! Symbol- and name-storage facilities.
//!
//! This is based on [Servo's string-cache
//! library](https://github.com/servo/string-cache); at the moment it's very
//! lifetime-unsafe for `Pooled` instances.
use std;
use std::fmt;
use std::convert::AsRef;
use std::marker::PhantomData;
use std::hash::{Hash, Hasher, SipHasher};
use std::collections::BTreeMap;
use std::borrow::ToOwned;

use super::util::memcpy;
use super::util::hash::{stdhash as hash};


#[macro_use]
pub mod nameable;
pub use self::nameable::Nameable;


/* ****************************************************************
 * Pool
 */
/// A collection of symbols.
#[derive(Clone, Debug)]
pub struct Pool {
    map:  BTreeMap<u64,String>
}

impl Pool {
    /// Create a new symbol pool.
    pub fn new() -> Pool {
        Pool{map: BTreeMap::new()}
    }

    /// Fetch a symbol corresponding to the given string.
    pub fn sym<'a>(&'a mut self, name: &str) -> Sym<'a> {
        Sym(unsafe { self.symbol(name) }, PhantomData)
    }

    /// Fetch a lifetime-unsafe symbol corresponding to the given string.
    ///
    /// Care must be taken to ensure that the returned symbol is not used after
    /// the pool goes out of scope.
    pub unsafe fn symbol(&mut self, name: &str) -> Symbol {
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

impl Default for Pool {
    fn default() -> Self {
        Pool{map: Default::default()}
    }
}

/* ****************************************************************
 * Symbol
 */
/// Lifetime-safe Symbol wrapper.
///
/// Instances of `Sym` are created using the `sym` method on
/// [`Pool`](struct.Pool.html).
///
/// `Sym` is a safe wrapper around `Symbol`, which does not carry a lifetime
/// parameter and therefore cannot ensure that any pool reference carried by an
/// instance is still valid.  By adding a lifetime parameter and appropriate
/// `PhantomData` data member, `Sym` prevents this case.
///
/// # Examples
///
/// The following should fail to compile -- as we expect it to in safe
/// Rust code.
///
/// ```rust,compile_fail
/// use std::mem;
/// use cripes::symbol::{Sym, Pool};
///
/// /// Return a Sym from a temporary Pool object.  This causes a compile error
/// /// because the pool is dropped at the end of the function.
/// fn make_sym<'a>(s: &'a str) -> Sym<'a> {
///     Pool::new().sym(s)
/// }
///
/// fn main() {
///     let s = make_sym("he who smelt it, dealt it");
///     println!("s = {}", s);
/// }
/// ```

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct Sym<'a>(Symbol, PhantomData<&'a Pool>);

impl<'a> fmt::Display for Sym<'a> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <Symbol as fmt::Display>::fmt(&self.0, f)
    }
}

impl<'a> AsRef<str> for Sym<'a> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl<'a> Hash for Sym<'a>  {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}



/// An atomic `Copy`able string.  Symbol either encodes a short string
/// directly, or stores it in an external Pool.
///
/// **Note that Symbol is unsafe to use** in situations where an instance may
/// outlive the pool that created it!  Its primary purpose is to
#[derive(Clone,Copy,Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct Symbol {
    value: [u64; 2],
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
    #[inline]
    fn name(&self) -> Symbol { *self }
}

impl Hash for Symbol {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state);
    }
}


impl fmt::Display for Symbol {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

// ================================================================
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
 * Types and tags
 */
const TAG_MASK: u8 = 0x01;
const POOLED: u8 = 0;
const INLINE: u8 = 1;

/// Storage type for a symbol instance.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum Type {
    /// Symbol data packed into the structure itself
    INLINE = 0,
    /// Reference to a value in a Pool
    POOLED = 1 }


/* ****************************************************************
 * Inline
 */
// We currently get
//
//    error: array length constant evaluation error: unimplemented constant expression: calling non-local const fn [E0250]
//
// when trying to declare the data array in `struct Inline`, below, if we try to use the obvious expression here.
//const INLINE_SYMBOL_MAX_LEN: usize = std::mem::size_of::<Symbol>() - 1;
/// Maximum number of bytes in a packed (inline) symbol name.
const INLINE_SYMBOL_MAX_LEN: usize = 15;


/// Data format for symbols packed entirely into a `Symbol` instance.
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Inline{len: u8, data: [u8; INLINE_SYMBOL_MAX_LEN]}

impl Inline {
    /// Create a new Inline-format symbol using the given string.
    pub fn new(name: &str) -> Self {
        panic_unless!(name.len() <= INLINE_SYMBOL_MAX_LEN,
                      "Argument to Inline::new (length {}) exceeds maximum length ({}) for an inlined symbol",
                      name.len(), INLINE_SYMBOL_MAX_LEN);
        let mut buf: [u8; INLINE_SYMBOL_MAX_LEN] = [0; INLINE_SYMBOL_MAX_LEN];

        unsafe { memcpy(&mut buf[..], name.as_bytes()); }
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

/* ****************************************************************
 * Pooled
 */
/// Data format for symbols stored as references to a symbol pool.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Pooled{key: u64, pool: *const Pool}

impl Pooled {
    #[inline(always)]
    fn new(key: u64, pool: *const Pool) -> Self {
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

/* ****************************************************************
 * Unpacked
 */

/// A differentiated `Symbol`.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
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
        match self {
            &Unpacked::Inline(ref x) => x.as_ref(),
            &Unpacked::Pooled(ref x) => x.as_ref()
        }
    }
}
