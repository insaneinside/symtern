// Copyright (C) 2016 Symtern Project Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-Apache
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. This file may not be copied, modified, or
// distributed except according to those terms.
//! "Lifetime-safe" interner adaptor.
// [Module documentation lives on the exported adaptor, `Luma`.]
use std::marker::PhantomData;
use std::cell::{RefCell, Ref};

use crate::{sym, traits, Result};

/// Symbol type used by the [`Luma`](struct.Luma.html) adaptor.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Sym<'a, W> {
    wrapped: W,
    lifetime: PhantomData<&'a ()>,
}

impl<'a,W> sym::Symbol for Sym<'a, W>
    where W: sym::Symbol {
    type Id = W::Id;

    #[cfg(debug_assertions)]
    fn pool_id(&self) -> sym::PoolId {
        self.wrapped.pool_id()
    }

    fn id(&self) -> Self::Id {
        self.wrapped.id()
    }

    fn id_ref(&self) -> &Self::Id {
        self.wrapped.id_ref()
    }

    #[cfg(debug_assertions)]
    fn create(id: Self::Id, pool_id: sym::PoolId) -> Self {
        Sym{wrapped: W::create(id, pool_id),
            lifetime: PhantomData}
    }

    /// Create a new value with the given ID.
    #[cfg(not(debug_assertions))]
    fn create(id: Self::Id) -> Self {
        Sym{wrapped: W::create(id),
            lifetime: PhantomData}
    }
}

impl<'a,W> From<W> for Sym<'a, W> {
    fn from(w: W) -> Self {
        Sym{wrapped: w, lifetime: PhantomData}
    }
}

/// "Lifetime-safe" interner adaptor.
///
/// This adaptor's symbols are treated as references to their source pool
/// &mdash; which, thanks to Rust's borrow-checker, prevents us from dropping
/// the pool before any of its symbols.
///
/// To achieve this, we utilize interior mutability via `RefCell` (which allows
/// us to intern more than one value at a time).  Note that this is _not_
/// a zero-cost abstraction: the price we pay for interior mutability is
/// run-time borrow-checking!  More specifically, certain borrow errors that
/// would normally be caught at compile-time...
///
/// ```rust,compile_fail file="tests/compile-fail/cannot-intern-with-active-resolved-ref.rs"
/// use symtern::prelude::*;
/// use symtern::Pool;
///
/// let mut pool = Pool::<str, u32>::new();
/// let x = pool.intern("foo").expect("failed to intern a value");
/// let foo = pool.resolve(x).expect("failed to resolve the value we just interned");
/// assert_eq!("foo", foo);
///
/// let _ = pool.intern("bar").expect("failed to intern a value"); //~ ERROR cannot borrow `pool` as mutable because it is also borrowed as immutable
/// ```
///
/// ...become run-time errors when using the `Luma` adaptor:
///
/// ```rust,ignore file="tests/run-fail/luma-panics-on-intern-with-active-resolved-ref.rs"
/// use symtern::prelude::*;
/// use symtern::Pool;
/// use symtern::adaptors::Luma;
///
/// let pool = Luma::from(Pool::<str, u32>::new());
/// let x = pool.intern("foo").expect("failed to intern a value");
/// let foo = pool.resolve(x).expect("failed to resolve the value we just interned");
/// assert_eq!("foo", &*foo);
///
/// let _ = pool.intern("bar").expect("failed to intern a value"); //~ PANIC already borrowed: BorrowMutError
/// ```
#[derive(Default)]
pub struct Luma<W> {
    wrapped: RefCell<W>
}

impl<W> Luma<W> {
    /// Create a new, empty `Luma` instance.
    pub fn new() -> Self
        where W: Default
    {
        Luma{wrapped: W::default().into()}
    }
}

impl<W> From<W> for Luma<W> {
    fn from(w: W) -> Self {
        Luma{wrapped: w.into()}
    }
}

impl<'a, W, BS, BI: ?Sized> traits::Intern for &'a Luma<W>
    where for<'b> &'b mut W: traits::Intern<Symbol=BS, Input=BI>,
          BS: sym::Symbol + traits::Symbol
{
    type Input = BI;
    type Symbol = Sym<'a,BS>;

    fn intern(self, input: &Self::Input) -> Result<Self::Symbol> {
        let inner_result = self.wrapped.borrow_mut().intern(input);
        inner_result.map(From::from)
    }
}

impl<'a, W, BI, BO: ?Sized> traits::Resolve for &'a Luma<W>
    where for<'b> &'b W: traits::Resolve<Input=BI, Output=&'b BO>,
          BI: sym::Symbol + traits::Symbol,
          BO: 'a
{
    type Input = Sym<'a,BI>;
    type Output = Ref<'a,BO>;
    fn resolve(self, sym: Self::Input) -> Result<Self::Output> {
        Ok(Ref::map(self.wrapped.borrow(), |w| w.resolve(sym.wrapped).unwrap()))
    }
}

impl<W> traits::Len for Luma<W> where W: traits::Len {
    fn len(&self) -> usize {
        self.wrapped.borrow().len()
    }
    fn is_full(&self) -> bool {
        self.wrapped.borrow().is_full()
    }
    fn is_empty(&self) -> bool {
        self.wrapped.borrow().is_empty()
    }
}

    
#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::basic::Pool;
    use super::Luma;

    /// Check that we can, in fact, intern -- and subsequently resolve -- more
    /// than one value at a time.
    #[test]
    fn can_intern_multiple_value() {
        let luma = Luma::from(Pool::<u64, u8>::new());
        let a = luma.intern(&0u64).expect("failed to intern value");
        let b = luma.intern(&1u64).expect("failed to intern value");
        assert!(a != b);
        assert_eq!(0u64, *luma.resolve(a).unwrap());
        assert_eq!(1u64, *luma.resolve(b).unwrap());
    }
}
