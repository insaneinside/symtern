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

use std::rc;
use std::ops::Deref;
use std::hash::{Hash, Hasher};
use std::borrow::{Borrow, ToOwned};

use traits::{InternerMut, SymbolId, Resolver, UnsafeResolver};
use {Result, ErrorKind};


/// `std::rc::Rc` wrapper with some `std::borrow::Borrow` specializations to
/// allow better ergonomics.
#[derive(Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
pub struct Rc<T: ?Sized>(rc::Rc<T>);

impl<T> Clone for Rc<T>
    where rc::Rc<T>: Clone
{
    fn clone(&self) -> Self {
        Rc(self.0.clone())
    }
}

impl Borrow<str> for Rc<String> {
    fn borrow(&self) -> &str {
        Borrow::<String>::borrow(&self.0).borrow()
    }
}

impl<T> Borrow<T> for Rc<T> {
    fn borrow(&self) -> &T {
        self.0.borrow()
    }
}

impl<T> Deref for Rc<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

/// Symbol type used by [the `shash` module](index.html)'s
/// [`Interner`](../traits/trait.Interner.html) implementation.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Sym<I> {
    id: I
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
    lookup_vec: Vec<Rc<T::Owned>>
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

    /// Check if the number of interned symbols has reached the maximum allowed
    /// for the pool's ID type.
    pub fn is_full(&self) -> bool {
        // Symbol IDs range from 0 to M, where M is given by `I::max_value()`;
        // hence a pool containing N entries is full iff N == M + 1.
        let len = self.lookup_vec.len();
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


/// Hash an object using the given hasher type.
fn hash<H: Hasher + Default, T: Hash>(obj: &T) -> u64 {
    let mut hasher = H::default();
    obj.hash(&mut hasher);
    hasher.finish()
}

// InternerMut
impl<T: ?Sized, I> InternerMut<T> for Pool<T, I>
    where I: SymbolId,
          T: ToOwned + Eq + Hash,
          T::Owned: Eq + Hash + Borrow<T>,
          Rc<T::Owned>: Borrow<T>,
{
    type Symbol = Sym<I>;
    fn intern(&mut self, value: &T) -> Result<Self::Symbol> {
        let h = hash::<::fnv::FnvHasher,_>(value);
       if let Some(&id) = self.ids_map.get(h) {
            return Ok(Sym{id: id/*, marker: PhantomData*/});
        } else if self.is_full() {
            return Err(ErrorKind::PoolOverflow.into())
        } else {
           //let rc = Rc(value.to_owned().into());
            self.lookup_vec.push(value.to_owned());

            // We do not expect this conversion to fail, since the condition in
            // the previous branch (`is_full()`) checks if a new ID would be
            // a representable value.
            let id = I::from_usize(self.lookup_vec.len() - 1)
                .expect("Unexpected failure to convert symbol ID from usize");
            self.ids_map.insert(h, id);

            Ok(Sym{id: id})
        }
    }
}

// ----------------------------------------------------------------
// Resolver
impl<T: ?Sized, I> Resolver<T> for Pool<T, I>
    where T: ToOwned + Eq + Hash,
          Rc<T::Owned>: Borrow<T>,
          T::Owned: Eq + Hash,
          I: SymbolId
{
    type Stored = T;
    fn resolve(&self, sym: Self::Symbol) -> Result<&Self::Stored> {
        // We previously converted the ID _from_ a usize, so this conversion should _not_ fail.
        let idx = sym.id.to_usize().expect("Unexpected failure to convert symbol ID to usize");

        if self.lookup_vec.len() > idx {
            Ok(self.lookup_vec[idx].borrow())
        } else {
            Err(ErrorKind::NoSuchSymbol.into())
        }
    }
}
impl<T: ?Sized, I> UnsafeResolver<T> for Pool<T, I>
    where T: ToOwned + Eq + Hash,
          Rc<T::Owned>: Borrow<T>,
          T::Owned: Eq + Hash,
          I: SymbolId
{
    unsafe fn resolve_unchecked(&self, symbol: Self::Symbol) -> &Self::Stored {
        let idx = symbol.id.to_usize().expect("Unexpected failure to convert symbol ID to usize");
        self.lookup_vec.get_unchecked(idx).borrow()
    }
}
