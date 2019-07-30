// Copyright (C) 2016 Symtern Project Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-Apache
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. This file may not be copied, modified, or
// distributed except according to those terms.
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
//! Symbols are created by calling [`intern`] on [`Intern`] implementations.
//!
//! ```rust file="examples/create-resolve.rs" id="create"
//! // Nearly all functionality is in the trait implementations, so we simply
//! // glob-import the traits.
//! use symtern::prelude::*;
//! use symtern::Pool;
//! use symtern::adaptors::Inline;
//!
//!
//! let mut basic_pool = Pool::<str,u16>::new();
//! let cat = basic_pool.intern("Kibbles").expect("Failed to intern the cat");
//!
//! let mut inline_pool = Inline::<Pool<str, u64>>::new();
//! let dog = inline_pool.intern("Fido").expect("Failed to intern the dog");
//! ```
//!
//! ## Symbol resolution
//!
//! Symbols are resolved using the [`Resolve`] trait.  In many cases you can
//! simply pass the symbol to the interner's [`resolve`] method by value:
//!
//! ```rust,ignore file="examples/create-resolve.rs" id="resolve"
//! assert_eq!(Ok("Kibbles"), basic_pool.resolve(cat));
//! ```
//!
//! Some `Resolve` implementations, however, require a _reference_ to
//! the symbol:
//!
//! ```rust,ignore file="examples/create-resolve.rs" id="resolve_ref"
//! assert_eq!(Ok("Fido"), inline_pool.resolve(&dog));
//! ```
//!
//! You can tell the difference by inspect the [`Input`][Resolve::Input]
//! associated type on each [`Resolve`] implementation.
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
//! let mut interner = MyInterner::new();
//! let x = interner.intern("x");
//! let y = interner.intern("y");        //~ ERROR cannot borrow `interner` as mutable more than once at a time
//! ```
//!
//! This happens because rustc's borrowck sees that `intern` takes a mutable
//! reference to the `MyInterner` instance and returns a symbol with the same
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
//! [`intern`]: trait.Intern.html#tymethod.intern
//! [`Intern`]: trait.Intern.html
//! [`Resolve`]: trait.Resolve.html
//! [`resolve`]: trait.Resolve.html#tymethod.resolve
//! [Resolve::Input]: trait.Resolve.html#associatedtype.Input
//! [Scala's path-dependent types]: http://danielwestheide.com/blog/2013/02/13/the-neophytes-guide-to-scala-part-13-path-dependent-types.html
use std::hash::Hash;
use ::num_traits::{Bounded, Unsigned, FromPrimitive, ToPrimitive};

use super::Result;

// ----------------------------------------------------------------

/// Trait describing primitive types used as symbols' internal representations.
pub trait SymbolId: Copy + Eq + Hash + Bounded + Unsigned + FromPrimitive + ToPrimitive {}
impl<T> SymbolId for T where T: Copy + Eq + Hash + Bounded + Unsigned + FromPrimitive + ToPrimitive {}

/// Trait bounds for symbol (interned stand-in value) types.
pub trait Symbol: Copy + Eq + Hash {}
impl<T> Symbol for T where T: Copy + Eq + Hash {}

// ----------------------------------------------------------------

/// Primary interface for interner implementations.
///
/// In order to allow for implementations that require a lifetime parameter,
/// and to abstract over mutability requirements, this trait's methods take
/// `self` by value; for a given type `T`, the trait should implemented for
/// either `&'a T` or `&'a mut T`.
pub trait Intern {
    /// Type of value accepted by `intern`.
    type Input: ?Sized;

    /// Type used to represent interned values.
    type Symbol: Symbol + crate::sym::Symbol;

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
    fn intern(self, value: &Self::Input) -> Result<Self::Symbol>;
}

// ----------------------------------------------------------------

/// Interface trait for types that provide the ability to resolve a symbol into
/// its referent.
///
/// In order to allow for implementations that require a lifetime parameter,
/// this trait's methods take `self` by value; for a given type `T`, the trait
/// should implemented for `&'a T`.
pub trait Resolve {
    /// Type passed to the [`resolve`](#tymethod.resolve) method.
    type Input;

    /// Type stored by the interner and made available with `resolve`.
    type Output;

    /// Look up and return a reference to the value represented by a symbol, or
    /// an error if the symbol was not found.
    ///
    /// ```rust,ignore file="examples/create-resolve.rs" id="resolve-with-error-handling"
    /// let s = match some_pool.resolve(sym) {
    ///     Ok(s) => s,
    ///     Err(err) => return Err(MyErrorType::from(err)),
    /// };
    /// ```
    fn resolve(self, symbol: Self::Input) -> Result<Self::Output>;
}


/// Interface for resolvers that can provide faster symbol resolution at the
/// expense of guaranteed safety.
///
/// Like [`Resolve`], this trait's methods take `self` by value.
///
/// [`Resolve`]: trait.Resolve.html
pub trait ResolveUnchecked: Resolve {
    /// Resolve the given symbol into its referent, bypassing any
    /// validity checks.
    ///
    /// ```rust,ignore file="examples/create-resolve.rs" id="resolve_unchecked"
    /// let mut pool = Pool::<str, u8>::new();
    /// let sym = try!(pool.intern("abc"));
    ///
    /// assert_eq!("abc", unsafe { pool.resolve_unchecked(sym) });
    /// ```
    unsafe fn resolve_unchecked(self, symbol: Self::Input) -> Self::Output;
}


/// Trait for use with interners that can report the number of values
/// they contain.
pub trait Len {
    /// Fetch the number of values contained in the interner.
    fn len(&self) -> usize;

    /// Check if the number of interned symbols has reached the
    /// maximum allowed.
    fn is_full(&self) -> bool;

    /// Check if the interner is "empty", i.e. has zero stored values.
    fn is_empty(&self) -> bool;
}
