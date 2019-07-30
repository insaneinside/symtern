// Copyright (C) 2016 Symtern Project Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-Apache
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. This file may not be copied, modified, or
// distributed except according to those terms.
//! # Fast general-purpose interners for every use case
//!
//! Symtern provides a high-performance interner implementation applicable to
//! most use cases, and a small set of adaptors that add additional
//! functionality on top of this base implementation.
//!
//! ## Trait-guided implementation
//!
//! Symtern's types are implemented around a core set of traits that define the
//! ways you can interact with an interner; these traits are carefully designed
//! to avoid restricting the kinds of adaptors possible.
//!
//! ## Interners and adaptors
//!
//! The base interner, [`Pool`], is generic over the type of interned values,
//! and can be configured to use any of Rust's numeric primitives for symbol
//! IDs.  It is the recommended interner for most purposes.
//!
//! ```rust file="examples/intro.rs" id="basic"
//! // Import Symtern's traits, which allow us to use each interner the same way
//! // regardless of the underlying implementation.
//! use symtern::prelude::*;
//!
//! // Create a new pool that accepts `&str` arguments to `intern`, and uses
//! // `u8` as the backing representation for its symbol type.
//! let mut pool = symtern::Pool::<str,u8>::new();
//! if let (Ok(hello), Ok(world)) = (pool.intern("Hello"), pool.intern("World")) {
//!     assert!(hello != world);
//!
//!     assert_eq!(hello, hello);
//!     assert_eq!(Ok(hello), pool.intern("Hello"));
//!     assert_eq!(Ok("Hello"), pool.resolve(hello));
//!
//!     assert_eq!(world, world);
//!     assert_eq!(Ok(world), pool.intern("World"));
//!     assert_eq!(Ok("World"), pool.resolve(world));
//! }
//! ```
//!
//! ### Adaptors
//!
//! For an overview of the available adaptors, see the [`adaptors` module].
//!
//! ## More examples
//!
//! [Symbol types](traits/trait.Symbol.html) are `Copy`:  they can be passed by
//! value without resulting in a move.
//!
//! ```rust file="examples/symbols-are-copy.rs" preserve=["main"]
//! use symtern::prelude::*;
//! use symtern::Pool;
//!
//! /// Take ownership of a value, consuming it.
//! fn consume<T>(_: T) {}
//!
//! let mut pool = Pool::<str, u32>::new();
//! let sym = pool.intern("xyz").unwrap();
//! consume(sym);
//! println!("The symbol is still valid: {:?}", pool.resolve(sym));
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
//! [`Pool`]: struct.Pool.html
//! [`adaptors` module]: adaptors/index.html
//! [`traits` module]: traits/index.html
#![warn(missing_docs)]
extern crate num_traits;
#[cfg(feature = "fnv")] extern crate fnv;

#[macro_use] mod sym;
mod core;
mod error;

pub mod traits;
mod basic;
pub mod adaptors;
pub mod prelude;

pub use crate::error::{Result, Error, ErrorKind};
pub use crate::basic::{Pool, Sym};
