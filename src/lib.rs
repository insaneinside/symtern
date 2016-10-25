// Copyright (C) 2016 Symtern Project Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-Apache
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. This file may not be copied, modified, or
// distributed except according to those terms.
//! # Fast general-purpose interners for every use case
//!
//! Symtern provides high-performance interners applicable to most use cases
//! (because let's face it: it's one of the least exciting parts of writing
//! a parser).
//!
//!   * [`basic`] is generic over the type of interned values, and can be
//!     configured to use any of Rust's numeric primitives for symbol IDs.
//!     It is the recommended interner for most purposes.
//!
//!   * [`short`] is optimized for short strings, which are stored directly in
//!     the returned symbol when under a certain length.  If you expect to be
//!     working with many short strings, it may perform better than the
//!     `basic` interner.
//!
//!
//! Each of these modules defines a `Pool` type that implements
//! [`traits::InternerMut`].
//!
//! For a more detailed introduction to the concepts and terminology used in
//! the library, visit [the `traits` module].
//!
//! ## Examples
//!
//! [Symbol types](traits/trait.Symbol.html) are `Copy`:  they can be passed by
//! value without resulting in a move.
//!
//! ```rust file="examples/symbols-are-copy.rs" preserve=["main"]
//! use symtern::basic::Pool;
//! use symtern::traits::*;
//!
//! /// Take ownership of a value, consuming it.
//! fn consume<T>(_: T) {}
//!
//! fn main() {
//!     let mut pool = Pool::<str, u32>::new();
//!     let sym = pool.intern("xyz").unwrap();
//!     consume(sym);
//!     println!("The symbol is still valid: {:?}", pool.resolve(sym));
//! }
//! ```
//!
//! ## Caveat Emptor
//!
//! Because of the way symbol types in this crate are represented, a symbol
//! obtained by calling `intern` on one `Pool` instance can easily be identical
//! to a symbol obtained from a different instance of the same `Pool` type
//! &mdash; and will resolve without error (albeit incorrectly) on that
//! other pool!
//!
//! Present-day Rust affords us no easy way to fix this without incurring
//! additional runtime costs; see the discussion
//! [here](traits/index.html#strikechoosingstrike-chasing-our-guarantees) for
//! more information.
//!
//! When the crate is compiled in debug mode, an additional field is added to
//! all symbol instances to allow run-time detection of attempts to resolve
//! a symbol on the wrong resolver, and any such attempt will trigger a panic.
//!
//! [`traits::InternerMut`]: traits/trait.InternerMut.html
//! [`basic`]: basic/index.html
//! [`short`]: short/index.html
//! [the `traits` module]: traits/index.html
#![warn(missing_docs)]

extern crate num_traits;
#[cfg(feature = "fnv")] extern crate fnv;

#[macro_use] mod sym;
mod core;
mod error;
pub mod traits;
pub mod short;
pub mod basic;

pub use error::{Result, Error, ErrorKind};
