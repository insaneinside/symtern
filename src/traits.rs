//! Traits that define the interface for all string-interning implementations.
//!
//! Interners are used to provide trivially-comparable stand-ins for values
//! that are not trivially comparable, and to later retrieve the value
//! represented by each stand-in.  (If you don't need the original value after
//! interning, you should probably be using hash functions instead.)  This
//! library uses the term "symbol" to refer to a stand-in value.
//!
//!
//! # Symbol resolution
//!
//! Most interner implementations will handle resolution of symbols into their
//! referents from the same interner that created the symbols; however in some
//! cases the symbols themselves may carry enough information to perform this
//! resolution without an explicit reference to the interner.
//!
//!   * [`Resolver`] should be implemented by interner types that handle symbol
//!     resolution themselves.
//!
//!   * [`SelfResolver`] should be implemented by symbol types that can be
//!     resolved without explicit reference to an interner.
//!     Unlike `Resolver`, the `resolve` method on this trait _must
//!     not_ fail.
//!
//!
//! # Choosing our Guarantees
//!
//! In order to safely allow this round-trip behavior in an error, we'd *like*
//! to require that
//!
//!   1. symbols can only be resolved by a single interner instance, and
//!
//!   2. interners outlive or contain a copy of the original value, and outlive
//!      the created stand-in.
//!
//! If we enforce the latter condition naively using Rust's lifetime system as
//! as follows, however, we find we can only create one stand-in object at
//! a time!
//!
//! ```rust ignore file=tests/compile-fail/naive-lifetime-safety.rs
//! struct MyInterner {
//!     // ...
//! }
//! struct Sym<'a> {
//!     marker: ::std::marker::PhantomData<&'a ()>,
//!     // ...
//! }
//! impl MyInterner {
//!     fn new() -> Self {
//!         // ...
//!     }
//!     fn intern(&'a mut self, s: &str) -> Sym<'a> {
//!         // ...
//!     }
//! }
//!
//! fn main() {
//!     let mut interner = MyInterner::new();
//!     let x = interner.intern("x");
//!     let y = interner.intern("y");        //~ ERROR cannot borrow `interner` as mutable more than once at a time
//! }
//! ```
//!
//! This happens because rustc's borrowck sees that `intern` takes a mutable
//! reference to the MyInterner instance and returns a symbol with the same
//! lifetime, and infers that this symbol has (mutably) borrowed the interner
//! through the reference.  To fix this we can do one of the following.
//!
//!   * Remove the lifetime from `Sym`, and lose any lifetime-safety
//!     guarantees.  This option is acceptable when the interner is required
//!     for symbol resolution anyway.
//!
//!   * Change `intern` to take `&self` and instead employ interior mutability
//!     through `RefCell`, `Mutex`, or similar.
//!
//! There are traits defined in this module to support either option;
//! [`Interner`] employs interior mutability while [`InternerMut`] forgoes
//! lifetime safety.
//!
//!
//! [`Interner`]: trait.Interner.html
//! [`InternerMut`]: trait.InternerMut.html
//! [`Resolver`]: trait.Resolver.html
//! [`SelfResolver`]: trait.SelfResolver.html
//!
use ::num_traits::{Bounded, Unsigned, FromPrimitive, ToPrimitive};

use super::Result;


/// Trait describing primitive types used as symbols' internal representations.
/// This trait exists primarily for the benefit of interners in this library.
pub trait SymbolId: Copy + Eq + Bounded + Unsigned + FromPrimitive + ToPrimitive {}
impl<T> SymbolId for T where T: Copy + Eq + Bounded + Unsigned + FromPrimitive + ToPrimitive {}

// ----------------------------------------------------------------

/// Trait bounds for symbol (interned stand-in value) types.
pub trait Symbol: Copy + PartialEq {}
impl<T> Symbol for T where T: Copy + PartialEq<T> {}


/// Primary interface for interner implementations.
///
/// The type parameter `T` is the type accepted by the `intern` method;
pub trait Interner<'a, T: ?Sized> {
    /// Type used to represent interned values.
    type Symbol: 'a + Symbol;

    /// Fetch the symbol that corresponds to the given value.  If the value
    /// does not map to any existing symbol, create and return a new one.
    fn intern(&'a self, value: &T) -> Result<Self::Symbol>;
}

/// Primary interface for interner implementations that do *not* use interior
/// mutability.
pub trait InternerMut<T: ?Sized> {
    /// Type used to represent interned values.
    type Symbol: Symbol;

    /// Fetch the symbol that corresponds to the given value.  If the value
    /// does not map to any existing symbol, create and return a new one.
    fn intern(&mut self, value: &T) -> Result<Self::Symbol>;
}

/// Trait for implementation by interners that directly provide
/// symbol resolution.
pub trait Resolve<S: Symbol> {
    /// Type stored by the interner and made available with `resolve`.
    type Target: ?Sized;

    /// Look up and return a reference to the value represented by a symbol, or
    /// an error if the symbol was not found.
    fn resolve(&self, symbol: S) -> Result<&Self::Target>;
}


/// Interface for resolvers that can provide faster symbol resolution at the
/// expense of guaranteed safety.
pub trait ResolveUnchecked<S: Symbol>: Resolve<S> {
    /// Resolve the given symbol into its referent, bypassing any
    /// validity checks.
    unsafe fn resolve_unchecked(&self, symbol: S) -> &Self::Target;
}



/// Trait implemented by interners that require a reference to a symbol in
/// order to resolve it.
pub trait ResolveRef<S> where S: Symbol {
    /// Type stored by the interner and made available with `resolve_ref`.
    type Target: ?Sized;

    /// Look up and return a reference to the value represented by a symbol, or
    /// an error if the symbol was not found.
    fn resolve_ref<'a, 'b, 'c>(&'a self, symbol: &'b S) -> Result<&'c Self::Target>
        where 'a: 'c,
              'b: 'c;
}
