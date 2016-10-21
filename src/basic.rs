//! Simple hash-based interner generic over both the type of interned values
//! and the type used to represent symbol IDs.
//!
//! Symbols produced by the interner type in this module may outlive the
//! interners that produced them; any attempt to resolve such symbols on
//! a different interner will return an error.
//!
//! [`Pool`](struct.Pool.html) can intern any type that implements `ToOwned`,
//! `Eq`, and `Hash`, where its owned type (`ToOwned::Owned`) also implements
//! `Eq` and `Hash`.

use std::hash::Hash;
use std::borrow::{Borrow, ToOwned};

use traits::{InternerMut, SymbolId, Resolve, ResolveUnchecked};
use {core, sym, Result, ErrorKind};
use sym::Symbol;

make_sym! {
    pub Sym<I>:
    "Symbol type used by [the `basic` module](index.html)'s [`InternerMut`](../traits/trait.InternerMut.html) implementation.";
}

#[cfg(feature = "fnv")]
type HashMap<K, V> = ::fnv::FnvHashMap<K, V>;

#[cfg(not(feature = "fnv"))]
type HashMap<K, V> = ::std::collections::HashMap<K, V>;

/// Simple hash-based interner generic over interned type and with support for
/// configurable symbol ID type.  See [the module-level
/// documentation](index.html) for more information.
#[derive(Clone, Debug)]
pub struct Pool<T: ?Sized, I = usize>
    where T: ToOwned + Eq + Hash,
          T::Owned: Eq + Hash,
          I: SymbolId
{
    ids_map: HashMap<u64, I>,
    lookup_vec: Vec<T::Owned>
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

    /// Get the number of entries contained in the pool.
    pub fn len(&self) -> usize {
        self.lookup_vec.len()
    }

    /// Check if the number of interned symbols has reached the maximum allowed
    /// for the pool's ID type.
    pub fn is_full(&self) -> bool {
        // Symbol IDs range from 0 to M, where M is given by `I::max_value()`;
        // hence a pool containing N entries is full iff N == M + 1.
        let len = self.len();
        len >= 1 && len - 1 >= I::max_value().to_usize().expect("Unexpected failure to convert index type `max_value()` result to usize")
    }
}

// Default
impl<T: ?Sized, I> Default for Pool<T, I>
    where T: ToOwned + Eq + Hash,
          T::Owned: Eq + Hash,
          I: SymbolId
{
    fn default() -> Self {
        Pool{ids_map: Default::default(),
             lookup_vec: Default::default()}
    }
}

// InternerMut
impl<T: ?Sized, I> InternerMut<T> for Pool<T, I>
    where I: SymbolId,
          T: ToOwned + Eq + Hash,
          T::Owned: Eq + Hash + Borrow<T>,
{
    type Symbol = Sym<I>;
    fn intern(&mut self, value: &T) -> Result<Self::Symbol> {
        let key = core::hash::<T, core::DefaultHashAlgo>(value);
        if let Some(&id) = self.ids_map.get(&key) {
            return Ok(sym::create(id))
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

            Ok(sym::create(id))
        }
    }
}

// ----------------------------------------------------------------
// Resolve
impl<T: ?Sized, I> Resolve<Sym<I>> for Pool<T, I>
    where T: ToOwned + Eq + Hash,
          T::Owned: Eq + Hash + Borrow<T>,
          I: SymbolId
{
    type Target = T;
    fn resolve(&self, s: <Self as InternerMut<T>>::Symbol) -> Result<&Self::Target> {
        // We previously converted the ID _from_ a usize, so this conversion should _not_ fail.
        let idx = s.id().to_usize().expect("Unexpected failure to convert symbol ID to usize");

        if self.lookup_vec.len() > idx {
            Ok(self.lookup_vec[idx].borrow())
        } else {
            Err(ErrorKind::NoSuchSymbol.into())
        }
    }
}
impl<T: ?Sized, I> ResolveUnchecked<Sym<I>> for Pool<T, I>
    where T: ToOwned + Eq + Hash,
          T::Owned: Eq + Hash + Borrow<T>,
          I: SymbolId
{
    unsafe fn resolve_unchecked(&self, symbol: Sym<I>) -> &Self::Target {
        let idx = symbol.id().to_usize().expect("Unexpected failure to convert symbol ID to usize");
        self.lookup_vec.get_unchecked(idx).borrow()
    }
}
