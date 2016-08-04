//! Implementation details for the string-interning library.
use std;
use std::mem;
use std::fmt;
use std::hash::{Hash, Hasher};

use super::Pool;

// We currently get
//
//    error: array length constant evaluation error: unimplemented constant expression: calling non-local const fn [E0250]
//
// when trying to declare the data array in `struct Inline`, below, if we try to use the obvious expression here.
//const INLINE_SYMBOL_MAX_LEN: usize = std::mem::size_of::<Symbol>() - 1;
/// Maximum number of bytes in a packed (inline) symbol name.
pub const INLINE_SYMBOL_MAX_LEN: usize = 15;


/// An atomic `Copy`able string.  Symbol either encodes a short string
/// directly, or stores it in an external Pool.
///
/// **Note that Symbol is unsafe to use** in situations where an instance may
/// outlive the pool that created it!
///
/// You probably want [`Sym`](struct.Sym.html) instead.
#[derive(Clone,Copy,Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct Symbol {
    value: [u64; 2],
}

impl Symbol {
    /// Convert the symbol to its unpacked representation.
    #[inline(always)]
    #[allow(dead_code)]
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


/// Utility wrapper for `std::ptr::copy` that retains C's `memcpy`
/// argument-order semantics.
#[inline(always)]
unsafe fn memcpy_nonoverlapping(dest: &mut [u8], src: &[u8]) {
    std::ptr::copy_nonoverlapping(src.as_ptr(), dest.as_mut_ptr(),
                                  std::cmp::min(src.len(), dest.len()))
}

/* ****************************************************************
 * Inline
 */
/// Data format for symbols packed entirely into a `Symbol` instance.
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Inline{len: u8, data: [u8; INLINE_SYMBOL_MAX_LEN]}

impl Inline {
    /// Create a new Inline-format symbol using the given string.
    pub fn new(name: &str) -> Self {
        let name_bytes = name.as_bytes();
        let name_bytes_len = name_bytes.len();
        panic_unless!(name_bytes_len <= INLINE_SYMBOL_MAX_LEN,
                      "Argument to Inline::new ({} bytes) exceeds maximum length ({}) for an inlined symbol",
                      name_bytes_len, INLINE_SYMBOL_MAX_LEN);
        let mut buf: [u8; INLINE_SYMBOL_MAX_LEN] = [0; INLINE_SYMBOL_MAX_LEN];

        unsafe {
            memcpy_nonoverlapping(&mut buf[..], name_bytes);
        }

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
            let mut dest: &mut [u8; 16] = mem::transmute(&mut out.value);
            dest[0] = (self.len << 1) | INLINE;
            memcpy_nonoverlapping(&mut dest[1..], &self.data);
            out
        }
    }
    unsafe fn unpack(sym: &Symbol) -> Self {
        let mut out = Inline{len: 0, data: [0; INLINE_SYMBOL_MAX_LEN]};
        let src: &[u8;16] =
            mem::transmute(&sym.value);

        panic_unless!(src[0] & TAG_MASK == INLINE, "Invalid tag bit for inlined symbol!");
        out.len = src[0] >> 1;
        panic_unless!(out.len <= INLINE_SYMBOL_MAX_LEN as u8,
                      "Symbol length ({}) exceeds maximum ({}) for inlined symbol",
                      out.len, INLINE_SYMBOL_MAX_LEN);
        memcpy_nonoverlapping(&mut out.data[..], &src[1..(out.len as usize + 1)]);

        out
    }

    unsafe fn as_slice_from<'t>(sym: &'t Symbol) -> &'t str {
        let src: &[u8; 16] = mem::transmute(&sym.value);
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
#[doc(hidden)]
pub struct Pooled{key: u64, pool: *const Pool}

impl Pooled {
    #[inline(always)]
    pub fn new(key: u64, pool: *const Pool) -> Self {
        Pooled{key: key, pool: pool}
    }

}

impl AsRef<str> for Pooled {
    fn as_ref<'t>(&'t self) -> &'t str {
        unsafe {
            mem::transmute((*self.pool).map.lock()
                           .expect("Failed to lock symbol pool mutex for lookup")
                           [&self.key].as_str())
        }
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
               pool: mem::transmute(sym.value[1] as *const Pool)}
    }

    unsafe fn as_slice_from<'t>(sym: &'t Symbol) -> &'t str {
        panic_unless!(sym.value[0] & TAG_MASK as u64 == POOLED as u64,
                      "Invalid flag bit for pooled symbol");
        mem::transmute(mem::transmute::<_,&'t Pool>(sym.value[1] as *const Pool)
                       .map.lock().expect("Failed to lock symbol pool mutex for lookup")
                       [&sym.value[0]].as_str())
    }
}

/* ****************************************************************
 * Unpacked
 */

/// A differentiated `Symbol`.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
#[doc(hidden)]
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
