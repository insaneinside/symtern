//! Simple hash-based interner generic over both the type of interned values
//! and the type used to represent symbol IDs.
//!
//! Symbols produced by the interner type in this module may outlive the
//! interners that produced them; any attempt to resolve such symbols on
//! a different interner will return an error.
//!
//! [`Pool`] can intern any type that implements `ToOwned`, `Eq`, and `Hash`,
//! where its owned type (`ToOwned::Owned`) also implements `Eq` and `Hash`.
//!
//! ```rust file="examples/you-can-intern-anything.rs"
//! use symtern::traits::*;
//!
//! #[derive(Clone, Eq, PartialEq, Hash)]
//! struct WibbleWobble {
//!     whee: Vec<u32>
//! }
//!
//! let mut pool = symtern::basic::Pool::<_,u8>::new();
//! assert!(pool.intern(&WibbleWobble{whee: vec![1, 2, 3, 4, 5]}).is_ok());
//! ```
//!
//! [`Pool`]: struct.Pool.html
//! [`InternerMut`]: ../traits/trait.InternerMut.html
use std::hash::Hash;
use std::borrow::{Borrow, ToOwned};
#[cfg(debug_assertions)] use std::sync::atomic::{self, AtomicUsize, Ordering};

use traits::{InternerMut, SymbolId, Resolve, ResolveUnchecked};
use {core, Result, ErrorKind};
use sym::{Symbol as ISymbol, Pool as IPool};


#[cfg(debug_assertions)]
static NEXT_POOL_ID: AtomicUsize = atomic::ATOMIC_USIZE_INIT;


#[cfg(feature = "fnv")]
type HashMap<K, V> = ::fnv::FnvHashMap<K, V>;

#[cfg(not(feature = "fnv"))]
type HashMap<K, V> = ::std::collections::HashMap<K, V>;

make_sym! {
    pub Sym<I>:
    "Symbol type used by [the `basic` module](index.html)'s [`InternerMut`](../traits/trait.InternerMut.html) implementation.";
}

/// Simple hash-based interner generic over interned type and with support for
/// configurable symbol ID type.
///
/// See [the module-level documentation](index.html) for more information.
#[derive(Clone, Debug)]
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

    /// Check if the pool is "empty", i.e. has zero stored values.
    pub fn is_empty(&self) -> bool {
        self.lookup_vec.is_empty()
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

impl<T: ?Sized, I> ::sym::Pool for Pool<T, I>
    where T: ToOwned + Eq + Hash,
          T::Owned: Eq + Hash,
          I: SymbolId
{
    #[cfg(debug_assertions)]
    fn id(&self) -> ::sym::PoolId {
        self.pool_id
    }

    type Symbol = <Self as InternerMut<T>>::Symbol;
    #[cfg(not(debug_assertions))]
    fn create_symbol(&self, id: <Self::Symbol as ::sym::Symbol>::Id) -> Self::Symbol {
        Sym::create(id)
    }

    #[cfg(debug_assertions)]
    fn create_symbol(&self, id: <Self::Symbol as ::sym::Symbol>::Id) -> Self::Symbol {
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
impl<T: ?Sized, I> Resolve<<Pool<T, I> as InternerMut<T>>::Symbol> for Pool<T, I>
    where T: ToOwned + Eq + Hash,
          T::Owned: Eq + Hash + Borrow<T>,
          I: SymbolId
{
    type Target = T;

    fn resolve(&self, s: <Self as InternerMut<T>>::Symbol) -> Result<&Self::Target> {
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
impl<T: ?Sized, I> ResolveUnchecked<<Pool<T, I> as InternerMut<T>>::Symbol> for Pool<T, I>
    where T: ToOwned + Eq + Hash,
          T::Owned: Eq + Hash + Borrow<T>,
          I: SymbolId
{
    unsafe fn resolve_unchecked(&self, symbol: <Pool<T, I> as InternerMut<T>>::Symbol) -> &Self::Target {
        let idx = symbol.id().to_usize().expect("Unexpected failure to convert symbol ID to usize");
        self.lookup_vec.get_unchecked(idx).borrow()
    }
}
