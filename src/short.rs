// Copyright (C) 2016 Symtern Project Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-Apache
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. This file may not be copied, modified, or
// distributed except according to those terms.
//! String interner with configurable ID type, optimized for short strings.
//!
//! [`Pool`], the interner type implemented in this module, will encode any
//! string _shorter_ than the symbol-ID type *directly inside the symbol*;
//! strings of the same or greater size will be passed to some unspecified
//! back-end implementation.
//!
//! Simple benchmarks included with the crate indicate that this gives an
//! approximately 6x (82%) speedup over [`basic::Pool`] for strings small
//! enough to be inlined.
//!
//! The downside to using this module's interner is its capacity, which is also
//! "short": the current implementation uses one of the bits of a symbol's ID
//! to mark it as containing an inlined string, which halves the number of
//! addressable slots in the look-up table.  But symbols with inlined values
//! don't occupy any space in the pool, so this may be a net gain if you expect
//! your input to be dominated by short strings.
//!
//! ```rust file="examples/short.rs"
//! use symtern::traits::*;
//!
//! let mut pool = symtern::short::Pool::<u64>::new();
//! let hello = pool.intern("Hello").expect("failed to intern a value");
//! let world = pool.intern("World").expect("failed to intern a value");
//!
//! assert!(hello != world);
//!
//! assert_eq!((Ok("Hello"), Ok("World")),
//!            (pool.resolve_ref(&hello),
//!             pool.resolve_ref(&world)));
//!
//! // Since both "Hello" and "World" are short enough to be inlined, they
//! // don't take up any space in the pool.
//! assert_eq!(0, pool.len());
//! ```
//!
//! The internal `Pack` trait, which provides the inlining functionality, is
//! implemented for `u16`, `u32`, and `u64`; it will be implemented for `u128`
//! as well when support for [128-bit integers] lands.
//!
//! [`Pool`]: struct.Pool.html
//! [`basic::Pool`]: ../basic/struct.Pool.html
//! [128-bit integers]: https://github.com/rust-lang/rfcs/blob/master/text/1504-int128.md
//!
use std::{mem, str};

use traits::{InternerMut, Len, Resolve, ResolveRef, SymbolId};
use {ErrorKind, Result};
use basic;
use sym::{Symbol as ISymbol, Pool as IPool};

/// Interface used to pack strings into symbol-IDs.  Any implementations of
/// this trait *must* store inlined-string length in the most-significant
/// _byte_ of the implementing type.
#[doc(hidden)]
pub trait Pack: Sized + PartialOrd {
    /// Check if the value contains an inlined string slice.
    fn is_inlined(&self) -> bool {
        *self >= Self::msb_mask()
    }

    /// Get a mask for the most-significant-bit in the implementor.
    fn msb_mask() -> Self;

    /// Pack a string slice into an instance of the implementing type,
    /// returning `Some(packed_value)`, or `None` if the slice is too long.
    fn pack(s: &str) -> Option<Self>;

    /// Fetch a reference to the inlined string slice, if any.
    fn get_packed_ref(&self) -> Option<&str>;
}

/// Create a mask value for the most significant _bit_ in an $N-_byte_
/// unsigned integer.
macro_rules! msb_mask {
    ($T: tt, $N: expr) => ( (1 as $T) << ($N * 8 - 1) );
}

#[test]
fn test_msb_mask() {
    assert_eq!(1 << 7, msb_mask!(u8, 1));
    assert_eq!(1 << 15, msb_mask!(u16, 2));
    assert_eq!(1 << 31, msb_mask!(u32, 4));
    assert_eq!(1u64 << 63, msb_mask!(u64, 8));
}

macro_rules! impl_pack {
    ($T: tt, $N: expr) => {
        impl Pack for $T {
            fn msb_mask() -> Self {
                msb_mask!($T, $N)
            }

            #[cfg(target_endian = "little")]
            fn pack(s: &str) -> Option<Self> {
                if s.len() >= $N { return None; }

                let mut bytes = [0u8; $N];
                bytes[0..s.len()].copy_from_slice(s.as_ref());
                bytes[$N - 1] = s.len() as u8 | 0x80;

                Some(unsafe { mem::transmute(bytes) })
            }
            #[cfg(target_endian = "big")]
            fn pack(s: &str) -> Option<Self> {
                if s.len() >= $N { return None; }

                let mut bytes = [0u8; $N];
                bytes[1..(s.len() + 1)].copy_from_slice(s.as_ref());
                bytes[0] = s.len() as u8 | 0x80;

                Some(unsafe { mem::transmute(bytes) })
            }

            #[cfg(target_endian = "little")]
            fn get_packed_ref(&self) -> Option<&str> {
                if ! self.is_inlined() { return None; }
                unsafe {
                    let bytes: &[u8; $N] = mem::transmute(self);
                    let len = (bytes[$N - 1] & ! 0x80) as usize;
                    Some(str::from_utf8_unchecked(&bytes[0..len]))
                }
            }
            #[cfg(target_endian = "big")]
            fn get_packed_ref(&self) -> Option<&str> {
                if ! self.is_inlined() { return None; }
                unsafe {
                    let bytes: &[u8; $N] = mem::transmute(self);
                    let len = (bytes[0] & ! 0x80) as usize;
                    match str::from_utf8_unchecked(&bytes[1..(len + 1)]) {
                        Ok(s) => Some(s),
                        Err(_) => None
                    }
                }
            }
        }
    }
}
impl_pack!(u16, 2);
impl_pack!(u32, 4);
impl_pack!(u64, 8);


make_sym! {
    pub Sym<I: Pack>(basic::Sym<I>):
    "Symbol type used by the [`short` module](index.html)'s [`InternerMut`](../traits/trait.InternerMut.html) implementation.";
}

/// Interner optimized for short strings.
///
/// See [the module-level documentation](index.html) for more information.
pub struct Pool<I>
    where I: SymbolId
{
    backend: basic::Pool<str, I>
}

impl<I> Pool<I>
    where I: SymbolId
{
    /// Create a new, empty symbol pool
    pub fn new() -> Self {
        Pool{backend: basic::Pool::new()}
    }
}

impl<B> Len for Pool<B>
    where B: Len,
          B::Symbol: sym::Symbol,
          <B::Symbol as sym::Symbol>::Id: Pack + ToPrimitive
{
    /// Fetch the number of items contained in the pool.  The returned value
    /// does not count values inlined in symbols.
    fn len(&self) -> usize {
        self.backend.len()
    }

    /// Check if the pool is "empty", i.e. has zero stored values.
    ///
    /// Because strings inlined in symbols are not stored in the pool, they do
    /// not affect the result of this method.
    fn is_empty(&self) -> bool {
        self.backend.is_empty()
    }

    /// Check if the number of interned symbols has reached the maximum allowed
    /// for the pool's ID type.
    fn is_full(&self) -> bool {
        self.backend.len() >= <<B::Symbol as sym::Symbol>::Id as Pack>::msb_mask().to_usize().unwrap()
    }
}

impl<I> ::sym::Pool for Pool<I>
    where I: SymbolId + Pack
{
    #[cfg(debug_assertions)]
    fn id(&self) -> ::sym::PoolId {
        self.backend.id()
    }

    type Symbol = <Self as InternerMut<str>>::Symbol;
    fn create_symbol(&self, id: <Self::Symbol as ::sym::Symbol>::Id) -> Self::Symbol {
        self.backend.create_symbol(id).into()
    }
}


impl<I> InternerMut<str> for Pool<I>
    where I: SymbolId + Pack
{
    type Symbol = Sym<I>;

    fn intern(&mut self, s: &str) -> Result<Self::Symbol> {
        match I::pack(s) {
            Some(id) => Ok(self.create_symbol(id)),
            None => {
                if self.is_full() {
                    Err(ErrorKind::PoolOverflow.into())
                } else {
                    match self.backend.intern(s) {
                        Ok(b) => Ok(b.into()),
                        Err(e) => Err(e)
                    }
                }
            }
        }
    }
}


impl<I> ResolveRef<Sym<I>> for Pool<I>
    where I: SymbolId + Pack
{
    type Target = str;
    fn resolve_ref<'a, 'b, 'c>(&'a self, symbol: &'b Sym<I>) -> Result<&'c Self::Target>
        where 'a: 'c,
              'b: 'c
    {
        match symbol.id_ref().get_packed_ref() {
            Some(s) => Ok(s),
            None => self.backend.resolve(symbol.wrapped)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Pool, Pack};
    use sym::Symbol;
    use traits::{InternerMut, ResolveRef};

    /// Check that the pool's size is affected only by non-inlined values.
    #[test]
    fn inlined_values_do_not_affect_size() {
        let mut pool = Pool::<u16>::new();
        assert!(pool.is_empty());

        // Inlined values shouldn't contribute to the pool's size.
        let x = pool.intern("x").expect("failed to intern single-character string");
        assert_eq!(0, pool.len());
        assert!(x.id().is_inlined());
        assert_eq!(Ok("x"), pool.resolve_ref(&x));

        let xy = pool.intern("xy").expect("failed to intern two-character string");
        assert_eq!(1, pool.len());
        assert!(! xy.id().is_inlined());
        assert_eq!(Ok("xy"), pool.resolve_ref(&xy));
    }

    /*/// Check that a `short` pool reports itself as full at the expected size.
    #[test]
    fn has_expected_capacity() {
        // FIXME: [bug] To fill the minimum-capacity pool (Pool<u16>) to
        // capacity, we need to generate 32768 unique string values of length
        // two or greater; it sure would be nice if we could find a crate to
        // help with this.
    }*/
}
