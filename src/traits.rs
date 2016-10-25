//! Traits that define the interface for all string-interning implementations.
//!
//! Interners are used to provide trivially-comparable stand-ins for values
//! that are not trivially comparable, and to later retrieve the value
//! represented by each stand-in.  (If you don't need the original value after
//! interning, you should probably be using hash functions instead.)
//!
//! ## Terminology
//!
//! This library uses the term "symbol" to refer to a stand-in value.
//! Interners, which both intern and resolve symbols, are called pools (in
//! reference to a "pool" of symbols) when discussing concrete
//! interner/resolver implementations.
//!
//! ## Symbol creation
//!
//! Symbols are created with the `intern` method on [`Interner`] or
//! [`InternerMut`] implementations.  (At the present time, this crate contains
//! only `InternerMut` implementations.)
//!
//! ```rust file="examples/create-resolve.rs" id="create"
//! // Nearly all functionality is in the trait implementations, so we simply
//! // glob-import the traits.
//! use symtern::traits::*;
//!
//! let mut basic_pool = symtern::basic::Pool::<str,u16>::new();
//! let cat = basic_pool.intern("Kibbles").expect("Failed to intern the cat");
//!
//! let mut short_pool = symtern::short::Pool::<u64>::new();
//! let dog = short_pool.intern("Fido").expect("Failed to intern the dog");
//! ```
//!
//! ## Symbol resolution
//!
//! The method used to resolve a symbol into its referent depends on the
//! resolver trait implemented by a the interner being used.  With [`Resolve`]
//! implementations, you simply call the interner's [`resolve`] method:
//!
//! ```rust,ignore file="examples/create-resolve.rs" id="resolve"
//! assert_eq!(Ok("Kibbles"), basic_pool.resolve(cat));
//! ```
//!
//! [`ResolveRef`] implementations, for whatever reason, instead of taking
//! a symbol by value require a _reference_ to the symbol.  This trait's method
//! is called [`resolve_ref`] to avoid potential confusion over the different
//! syntax required when passing an argument.
//!
//! ```rust,ignore file="examples/create-resolve.rs" id="resolve_ref"
//! assert_eq!(Ok("Fido"), short_pool.resolve_ref(&dog));
//! ```
//!
//! ## <strike>Choosing</strike> Chasing our Guarantees
//!
//! Few people enjoy dealing with the fact that software is fallible and
//! operations may fail, but doing so is often a necessary chore.  Sometimes,
//! however, we can eliminate potential error conditions through careful design
//! of an API.  How would we go about doing this for an interner library like
//! Symtern?  We'd like to require that
//!
//!   1. interning never fails, i.e. an interner grows as necessary to hold its
//!      values;
//!
//!   2. resolution never fails:
//!
//!     a. any attempt to resolve a symbol using an interner that did not
//!        create it results in a compile-time error; and
//!
//!     b. interners outlive or contain a copy of the original value, and outlive
//!        the created stand-in.
//!
//! The first condition is untenable for the library's expected use case in
//! combination with practical concerns of performance and efficiency, since
//! a progressively larger address space will require progressively larger
//! representations for symbols.  We *must* handle the address-space-exhausted
//! condition &mdash; especially if our symbols are 32 bits or fewer.
//!
//! There is no way to enforce the second condition (2a) in the general case
//! using features available in present-day Rust.  We can restrict which _type_
//! of wrong interner one can attempt to resolve the symbol on, but without
//! something like [Scala's path-dependent types] we can't generally enforce
//! the condition using compile-time checks.[¹](#footnote-1) What's worse, we
//! can't even promise zero-cost run-time checks!
//!
//! The third condition (2b), at least, can be satisfied using Rust's lifetime
//! system.  With a naive implementation, however, we find we can only create
//! one stand-in object at a time!
//!
//! ```rust,ignore file="tests/compile-fail/naive-lifetime-safety.rs"
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
//!     fn intern<'a>(&'a mut self, s: &str) -> Sym<'a> {
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
//!   * Change `intern` to take `&self` and instead employ interior mutability
//!     through `RefCell`, `Mutex`, or similar.
//!
//!   * Remove the lifetime from `Sym`, and lose any lifetime-safety
//!     guarantees.
//!
//! ### Vacuous Promises and Viable Features
//!
//! As we've just seen, we *are* going to be doing error handling for both
//! symbol creation *and* symbol resolution unless we heavily restrict or
//! change the ways in which the library can be used.  But maybe we can employ
//! the lifetime-safety guarantee in such a way that the benefits outweigh the
//! burden of requiring implementations to use interior mutability.
//!
//! One example is a method that clears (resets) an interner:
//!
//! ```rust,ignore
//!     fn intern<'a>(&'a self, s: &str) -> Sym<'a> { /* ... */ }
//!     fn clear(&mut self) { /* ... */ }
//! ```
//!
//! Because `clear` borrows `self` mutably, it cannot be called until all
//! immutable borrows held by `Sym<'a>` instances have ended.
//!
//! ## Footnotes
//!
//! <a name="footnote-1">¹</a>: We *could* produce symbols tied to the source
//! interner, but the method of doing so would impose severe restrictions on
//! the ways the library could be used.  Using an approach called
//! "generativity", symbols would be valid only within the scope of a closure
//! passed as a second argument to `intern` , as exemplified in the
//! [indexing] crate.
//!
//! [indexing]: https://github.com/bluss/indexing
//! [`Interner`]: trait.Interner.html
//! [`InternerMut`]: trait.InternerMut.html
//! [`Resolve`]: trait.Resolve.html
//! [`resolve`]: trait.Resolve.html#tymethod.resolve
//! [`ResolveRef`]: trait.ResolveRef.html
//! [`resolve_ref`]: trait.ResolveRef.html#tymethod.resolve_ref
//! [Scala's path-dependent types]: http://danielwestheide.com/blog/2013/02/13/the-neophytes-guide-to-scala-part-13-path-dependent-types.html
use ::num_traits::{Bounded, Unsigned, FromPrimitive, ToPrimitive};

use super::Result;

// ----------------------------------------------------------------

/// Trait describing primitive types used as symbols' internal representations.
pub trait SymbolId: Copy + Eq + Bounded + Unsigned + FromPrimitive + ToPrimitive {}
impl<T> SymbolId for T where T: Copy + Eq + Bounded + Unsigned + FromPrimitive + ToPrimitive {}

/// Trait bounds for symbol (interned stand-in value) types.
pub trait Symbol: Copy + PartialEq {}
impl<T> Symbol for T where T: Copy + PartialEq<T> {}

// ----------------------------------------------------------------

/// Primary interface for interner implementations that make use of
/// interior mutability.
///
/// This trait is not currently used by any interner implemented in the crate,
/// and may be removed prior to release; see the discussion
/// [here](index.html#strikechoosingstrike-chasing-our-guarantees).
pub trait Interner<'a, T: ?Sized> {
    /// Type used to represent interned values.
    type Symbol: 'a + Symbol;

    /// Fetch the symbol that corresponds to the given value.  If the value
    /// does not map to any existing symbol, create and return a new one.
    /// This method may return an error if the interner is out of space.
    fn intern(&'a self, value: &T) -> Result<Self::Symbol>;
}

/// Primary interface for interner implementations.
///
/// This trait may be renamed to [`Interner`](trait.Interner.html) prior to
/// release; see the discussion
/// [here](index.html#strikechoosingstrike-chasing-our-guarantees).
pub trait InternerMut<T: ?Sized> {
    /// Type used to represent interned values.
    type Symbol: Symbol;

    /// Fetch the symbol that corresponds to the given value.  If the value
    /// does not map to any existing symbol, create and return a new one.
    /// This method may return an error if the interner encounters any error
    /// while storing the value.
    ///
    /// ```rust,ignore file="examples/create-resolve.rs" id="intern-with-error-handling"
    /// let symbol = match some_interner.intern("Rosebud") {
    ///     Ok(sym) => sym,
    ///     Err(err) => return Err(MyErrorType::from(err)),
    /// };
    /// ```
    fn intern(&mut self, value: &T) -> Result<Self::Symbol>;
}

// ----------------------------------------------------------------

/// Trait for implementation by interners that directly provide
/// symbol resolution.
pub trait Resolve<S: Symbol> {
    /// Type stored by the interner and made available with `resolve`.
    type Target: ?Sized;

    /// Look up and return a reference to the value represented by a symbol, or
    /// an error if the symbol was not found.
    ///
    /// ```rust,ignore file="examples/create-resolve.rs" id="resolve-with-error-handling"
    /// let s = match some_pool.resolve(sym) {
    ///     Ok(s) => s,
    ///     Err(err) => return Err(MyErrorType::from(err)),
    /// };
    /// ```
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
    ///
    /// ```rust,ignore file="examples/create-resolve.rs" id="resolve_ref-with-error-handling"
    /// let s = match some_pool.resolve_ref(&sym) {
    ///     Ok(s) => s,
    ///     Err(err) => return Err(MyErrorType::from(err)),
    /// };
    /// ```
    fn resolve_ref<'a, 'b, 'c>(&'a self, symbol: &'b S) -> Result<&'c Self::Target>
        where 'a: 'c,
              'b: 'c;
}
