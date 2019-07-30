// Copyright (C) 2016 Symtern Project Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-Apache
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. This file may not be copied, modified, or
// distributed except according to those terms.
//! Basic hash-based generic interner.

use std::hash::Hash;
use std::borrow::{Borrow, ToOwned};
#[cfg(debug_assertions)] use std::sync::atomic::{self, AtomicUsize, Ordering};

use crate::traits::{Intern, Resolve, ResolveUnchecked, Len, SymbolId};
use crate::{core, Result, ErrorKind};
use crate::sym::{Symbol as ISymbol, Pool as IPool};


#[cfg(debug_assertions)]
static NEXT_POOL_ID: AtomicUsize = atomic::AtomicUsize::new(0);


#[cfg(feature = "fnv")]
type HashMap<K, V> = ::fnv::FnvHashMap<K, V>;

#[cfg(not(feature = "fnv"))]
type HashMap<K, V> = ::std::collections::HashMap<K, V>;

make_sym! {
    pub Sym<I>:
    "Symbol type used by [`Pool`](struct.Pool.html)'s [`Intern`](../traits/trait.Intern.html) and [`Resolve`](../traits/trait.Resolve.html) implementations.";
}

/// Simple hash-based interner generic over both the type of interned values
/// and the type used to represent symbol IDs.
///
/// `Pool` can intern any type that implements `ToOwned`, `Eq`, and `Hash`,
/// where its owned type (`ToOwned::Owned`) also implements `Eq` and `Hash`.
///
/// ```rust file="examples/you-can-intern-anything.rs"
/// use symtern::prelude::*;
/// use symtern::Pool;
///
/// #[derive(Clone, Eq, PartialEq, Hash)]
/// struct WibbleWobble {
///     whee: Vec<u32>
/// }
///
/// let mut pool = Pool::<_,u8>::new();
/// assert!(pool.intern(&WibbleWobble{whee: vec![1, 2, 3, 4, 5]}).is_ok());
/// ```
#[derive(Debug)]
pub struct Pool<T: ?Sized, I = usize>
    where T: ToOwned + Eq + Hash,
          T::Owned: Eq + Hash,
          I: SymbolId
{
    ids_map: HashMap<u64, I>,
    lookup_vec: Vec<T::Owned>,
    #[cfg(debug_assertions)]
    pool_id: usize
}

impl<T: ?Sized, I> Clone for Pool<T, I>
    where T: ToOwned + Eq + Hash,
          T::Owned: Eq + Hash + Clone,
          I: SymbolId,
{
    #[cfg(debug_assertions)]
    fn clone(&self) -> Self {
        Pool{ids_map: self.ids_map.clone(),
             lookup_vec: self.lookup_vec.clone(),
             pool_id: self.pool_id}
    }
    #[cfg(not(debug_assertions))]
    fn clone(&self) -> Self {
        Pool{ids_map: self.ids_map.clone(),
             lookup_vec: self.lookup_vec.clone()}
    }
}

// (inherent impl)
impl<T: ?Sized, I> Pool<T, I>
    where T: ToOwned + Eq + Hash,
          T::Owned: Eq + Hash,
          I: SymbolId
{
    /// Create a new, empty `Pool` instance.
    pub fn new() -> Self {
        Default::default()
    }
}

impl<'a, T: ?Sized, I> Len for Pool<T, I>
    where T: ToOwned + Eq + Hash,
          T::Owned: Eq + Hash,
          I: SymbolId
{
    /// Get the number of entries contained in the pool.
    fn len(&self) -> usize {
        self.lookup_vec.len()
    }

    /// Check if the pool is "empty", i.e. has zero stored values.
    fn is_empty(&self) -> bool {
        self.lookup_vec.is_empty()
    }

    /// Check if the number of interned symbols has reached the maximum allowed
    /// for the pool's ID type.
    fn is_full(&self) -> bool {
        // Symbol IDs range from 0 to M, where M is given by `I::max_value()`;
        // hence a pool containing N entries is full iff N == M + 1.
        let len = self.len();
        len >= 1 && len - 1 >= I::max_value().to_usize().expect("Unexpected failure to convert index type `max_value()` result to usize")
    }
}

impl<'a, T: ?Sized, I> crate::sym::Pool for Pool<T, I>
    where T: ToOwned + Eq + Hash,
          T::Owned: Eq + Hash,
          I: SymbolId
{
    type Symbol = Sym<I>;

    #[cfg(debug_assertions)]
    fn id(&self) -> crate::sym::PoolId {
        self.pool_id
    }

    #[cfg(not(debug_assertions))]
    fn create_symbol(&self, id: <Self::Symbol as ::sym::Symbol>::Id) -> Self::Symbol {
        Sym::create(id)
    }

    #[cfg(debug_assertions)]
    fn create_symbol(&self, id: <Self::Symbol as crate::sym::Symbol>::Id) -> Self::Symbol {
        Sym::create(id, self.id())
    }
}

// Default
impl<T: ?Sized, I> Default for Pool<T, I>
    where T: ToOwned + Eq + Hash,
          T::Owned: Eq + Hash,
          I: SymbolId
{
    #[cfg(not(debug_assertions))]
    fn default() -> Self {
        Pool{ids_map: Default::default(),
             lookup_vec: Default::default()}
    }
    #[cfg(debug_assertions)]
    fn default() -> Self {
        Pool{ids_map: Default::default(),
             lookup_vec: Default::default(),
             pool_id: NEXT_POOL_ID.fetch_add(1, Ordering::SeqCst)}
    }
}

// Intern
impl<'a, T: ?Sized, I> Intern for &'a mut Pool<T, I>
    where I: SymbolId,
          T: ToOwned + Eq + Hash,
          T::Owned: Eq + Hash + Borrow<T>,
{
    type Input = T;
    type Symbol = Sym<I>;

    fn intern(self, value: &Self::Input) -> Result<Self::Symbol> {
        let key = core::hash::<T, core::DefaultHashAlgo>(value);
        if let Some(&id) = self.ids_map.get(&key) {
            return Ok(self.create_symbol(id))
        } else if self.is_full() {
            return Err(ErrorKind::PoolOverflow.into())
        } else {
            self.lookup_vec.push(value.to_owned());

            // We do not expect this conversion to fail, since the condition in
            // the previous branch (`is_full()`) checks if a new ID would be
            // a representable value.
            let id = I::from_usize(self.lookup_vec.len() - 1)
                .expect("Unexpected failure to convert symbol ID from usize");
            self.ids_map.insert(key, id);

            Ok(self.create_symbol(id))
        }
    }
}

#[cfg(debug_assertions)]
macro_rules! check_matching_pool {
    ($slf: ident, $sym: ident) => {
        if $sym.pool_id() != $slf.id() {
            panic!(concat!("\nDetected an invalid attempt to resolve a symbol on a pool that did not\n",
                           "create it.  This is a bug in the program or library using Symtern; do not\n",
                           "report it to the Symtern developers."));
        }
    };
}

#[cfg(not(debug_assertions))]
macro_rules! check_matching_pool {
    ($slf: ident, $sym: ident) => {};
}

// ----------------------------------------------------------------
// Resolve
impl<'a,T: ?Sized, I> Resolve for &'a Pool<T, I>
    where T: ToOwned + Eq + Hash,
          T::Owned: Eq + Hash + Borrow<T>,
          I: SymbolId
{
    type Input = <&'a mut Pool<T, I> as Intern>::Symbol;
    type Output = &'a T;

    fn resolve(self, s: Self::Input) -> Result<Self::Output> {
        check_matching_pool!(self, s);
        // We previously converted the ID _from_ a usize, so this conversion should _not_ fail.
        let idx = s.id().to_usize().expect("Unexpected failure to convert symbol ID to usize");

        if self.lookup_vec.len() > idx {
            Ok(self.lookup_vec[idx].borrow())
        } else {
            Err(ErrorKind::NoSuchSymbol.into())
        }
    }
}
impl<'a, T: ?Sized, I> ResolveUnchecked for &'a Pool<T, I>
    where T: ToOwned + Eq + Hash,
          T::Owned: Eq + Hash + Borrow<T>,
          I: SymbolId
{
    unsafe fn resolve_unchecked(self, symbol: Self::Input) -> Self::Output {
        let idx = symbol.id().to_usize().expect("Unexpected failure to convert symbol ID to usize");
        self.lookup_vec.get_unchecked(idx).borrow()
    }
}


#[cfg(test)]
mod tests {
    use super::Pool;
    use crate::traits::*;
    use crate::ErrorKind;

    #[test]
    fn resolve_returns_expected_results() {
        let mut p1 = Pool::<str,u16>::new();
        let mut p2 = Pool::<str,u16>::new();

        let s1 = p1.intern("foo").unwrap();
        let s2 = p2.intern("bar").unwrap();

        assert_eq!(Ok("foo"), p1.resolve(s1));
        assert_eq!(Ok("bar"), p2.resolve(s2));
    }

    #[test]
    fn has_expected_len_and_capacity() {
        let mut pool = Pool::<u16,u8>::new();

        assert!(pool.is_empty());

        for i in 0u16..200 {
            pool.intern(&i).expect("failed to intern value");
        }
        assert_eq!(200, pool.len());
        assert!(! pool.is_full());

        for i in 150u16..250 {
            pool.intern(&i).expect("failed to intern value");
        }
        assert_eq!(250, pool.len());
        assert!(! pool.is_full());

        for i in 250u16..256 {
            pool.intern(&i).expect("failed to intern value");
        }
        assert_eq!(256, pool.len());
        assert!(pool.is_full());

        // The pool is full, but interning previously-interned values should
        // still result in Ok(_).
        pool.intern(&123).expect("failed to intern previously-interned value");
        match pool.intern(&456) {
            Ok(_) => panic!("unexpected `Ok` when interning unseen value in full pool"),
            Err(e) => assert_eq!(ErrorKind::PoolOverflow, e.kind()),
        }
    }
}
