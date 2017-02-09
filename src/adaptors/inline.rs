// Copyright (C) 2016-2017 Symtern Project Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-Apache
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. This file may not be copied, modified, or
// distributed except according to those terms.
//! Interner adaptor that uses the short-string optimization.
// [Module documentation lives on the exported adaptor, `Inline`.]
use std::{mem, str};

use num_traits::ToPrimitive;

use traits::{Intern, Resolve, Len};
use {ErrorKind, Result};
use sym::{self, Symbol, SymbolId};

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

/// Symbol type used by the [`Inline`](struct.Inline.html) adaptor.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Sym<S> {
    wrapped: S
}

impl<S> sym::Symbol for Sym<S>
    where S: sym::Symbol
{
    type Id = S::Id;

    #[cfg(debug_assertions)]
    fn pool_id(&self) -> ::sym::PoolId {
        self.wrapped.pool_id()
    }

    fn id(&self) -> Self::Id { self.wrapped.id() }
    fn id_ref(&self) -> &Self::Id { self.wrapped.id_ref() }
}

impl<S> sym::Create for Sym<S> where S: sym::Create {
    #[cfg(not(debug_assertions))]
    fn create(id: Self::Id) -> Self {
        Sym{wrapped: S::create(id)}
    }

    #[cfg(debug_assertions)]
    fn create(id: Self::Id, pool_id: ::sym::PoolId) -> Self {
        Sym{wrapped: S::create(id, pool_id)}
    }
}

impl<S> From<S> for Sym<S> {
    fn from(s: S) -> Self {
        Sym{wrapped: s}
    }
}

/// Interner adaptor optimized for short strings.
///
/// `Inline` will encode any string _shorter_ than the symbol-ID type *directly
/// inside the symbol*; strings of the same or greater size will be passed to
/// the wrapped interner.
///
/// Simple benchmarks included with the crate indicate that this gives an
/// approximately 6x (82%) speedup over the basic [`Pool`] for strings small
/// enough to be inlined.
///
/// The downside to using this adaptor is its capacity, which is also "short":
/// the current implementation uses one of the bits of a symbol's ID to mark it
/// as containing an inlined string, which halves the number of addressable
/// slots in the look-up table.  But symbols with inlined values don't occupy
/// any space in the pool, so this may be a net gain if you expect your input
/// to be dominated by short strings.
///
/// ```rust file="examples/short.rs"
/// use symtern::prelude::*;
/// use symtern::Pool;
/// use symtern::adaptors::Inline;
///
/// let mut pool = Inline::<Pool<str,u64>>::new();
/// let hello = pool.intern("Hello").expect("failed to intern a value");
/// let world = pool.intern("World").expect("failed to intern a value");
///
/// assert!(hello != world);
///
/// assert_eq!((Ok("Hello"), Ok("World")),
///            (pool.resolve(&hello),
///             pool.resolve(&world)));
///
/// // Since both "Hello" and "World" are short enough to be inlined, they
/// // don't take up any space in the pool.
/// assert_eq!(0, pool.len());
/// ```
///
/// The internal `Pack` trait, which provides the inlining functionality, is
/// implemented for `u16`, `u32`, and `u64`; it will be implemented for `u128`
/// as well when support for [128-bit integers] lands.
///
/// [`Pool`]: ../struct.Pool.html
/// [128-bit integers]: https://github.com/rust-lang/rfcs/blob/master/text/1504-int128.md
#[derive(Copy, Clone, Debug)]
pub struct Inline<W> {
    wrapped: W
}

impl<W> Inline<W> {
    /// Create a new, empty symbol pool
    pub fn new() -> Self
        where W: Default
    {
        Default::default()
    }
}

impl<W> Default for Inline<W>
    where W: Default
{
    fn default() -> Self {
        Inline{wrapped: Default::default()}
    }
}


impl<W> From<W> for Inline<W> {
    fn from(w: W) -> Self {
        Inline{wrapped: w}
    }
}

impl<W> Len for Inline<W>
    where W: Len + ::sym::Pool,
          <<W as sym::Pool>::Symbol as sym::Symbol>::Id: Pack + ToPrimitive
{
    /// Fetch the number of items contained in the pool.  The returned value
    /// does not count values inlined in symbols.
    fn len(&self) -> usize {
        (&self.wrapped).len()
    }

    /// Check if the pool is "empty", i.e. has zero stored values.
    ///
    /// Because strings inlined in symbols are not stored in the pool, they do
    /// not affect the result of this method.
    fn is_empty(&self) -> bool {
        (&self.wrapped).is_empty()
    }

    /// Check if the number of interned symbols has reached the maximum allowed
    /// for the pool's ID type.
    fn is_full(&self) -> bool {
        (&self.wrapped).len() >= <<<W as sym::Pool>::Symbol as sym::Symbol>::Id as Pack>::msb_mask().to_usize().unwrap()
    }
}

impl<W> ::sym::Pool for Inline<W>
    where W: sym::Pool,
          <<W as sym::Pool>::Symbol as sym::Symbol>::Id: Pack,
{
    type Symbol = W::Symbol;

    #[cfg(debug_assertions)]
    fn id(&self) -> ::sym::PoolId {
        self.wrapped.id()
    }

    fn create_symbol(&self, id: <<W as sym::Pool>::Symbol as ::sym::Symbol>::Id) -> Self::Symbol {
        <W as sym::Pool>::create_symbol(&self.wrapped, id).into()
    }
}


macro_rules! impl_intern {
    ($($mutt: tt)*) => {
        impl<'a, W, WS> Intern for &'a $($mutt)* Inline<W>
            where W: Len + sym::Pool<Symbol=WS>,
                  &'a $($mutt)* W: Intern<Input=str,Symbol=<W as sym::Pool>::Symbol>,
                  WS: sym::Symbol,
                  WS::Id: Pack
        {
            type Input = str;
            type Symbol = Sym<WS>;

            fn intern(self, s: &Self::Input) -> Result<Self::Symbol> {
                match WS::Id::pack(s) {
                    Some(id) => Ok(Sym{wrapped: self.wrapped.create_symbol(id)}),
                    None => {
                        // since max capacity is changed by this adaptor, we
                        // need to do a capacity-check here.
                        if self.is_full() {
                            Err(ErrorKind::PoolOverflow.into())
                        } else {
                            match self.wrapped.intern(s) {
                                Ok(b) => Ok(b.into()),
                                Err(e) => Err(e)
                            }
                        }
                    }
                }
            }
        }
    }
}
impl_intern!();
impl_intern!(mut);

impl<'a, 'sym, W, WS> Resolve<&'sym Sym<WS>> for &'a Inline<W>
    where 'sym: 'a,
          &'a W: sym::Pool<Symbol=WS> + Resolve<&'sym WS, Output=&'a str>,
          WS: sym::Symbol,
          WS::Id: Pack

{
    type Output = &'a str;

    fn resolve(self, symbol: &'sym Sym<WS>) -> Result<Self::Output>
    {
        match symbol.id_ref().get_packed_ref() {
            Some(s) => Ok(s),
            None => self.wrapped.resolve(&symbol.wrapped)
        }
    }
}


#[cfg(test)]
mod tests {
    use super::{Inline, Pack};
    use sym::Symbol;
    use traits::{Intern, Resolve, Len};

    /// Check that the pool's size is affected only by non-inlined values.
    #[test]
    fn inlined_values_do_not_affect_size() {
        let mut pool = Inline::<::basic::Pool<str,u16>>::new();
        assert!(pool.is_empty());

        // Inlined values shouldn't contribute to the pool's size.
        let x = pool.intern("x").expect("failed to intern single-character string");
        assert_eq!(0, pool.len());
        assert!(x.id().is_inlined());
        assert_eq!(Ok("x"), pool.resolve(&x));

        let xy = pool.intern("xy").expect("failed to intern two-character string");
        assert_eq!(1, pool.len());
        assert!(! xy.id().is_inlined());
        assert_eq!(Ok("xy"), pool.resolve(&xy));
    }

    /// Check that we can stack Inline adaptors and still resolve through
    /// them.  This is a compile-time check:  we're verifying that the Resolve
    /// implementation works whether the wrapped pool takes its `resolve`
    /// argument by value *or* by reference.
    #[cfg(feature="composition-tests")]
    #[test]
    fn can_stack_inliners() {
        let mut pool = Inline::<Inline<::basic::Pool<str,u16>>>::new();
        let xy = pool.intern("xy").expect("failed to intern two-character string");
        assert_eq!(Ok("xy"), pool.resolve(&xy));
    }

    /*/// Check that an `Inline` pool reports itself as full at the expected size.
    #[test]
    fn has_expected_capacity() {
        // FIXME: [bug] To fill the minimum-capacity pool (Inline<Pool<str,u16>) to
        // capacity, we need to generate 32768 unique string values of length
        // two or greater; it sure would be nice if we could find a crate to
        // help with this.
    }*/
}
