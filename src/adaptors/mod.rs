// Copyright (C) 2016 Symtern Project Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-Apache
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. This file may not be copied, modified, or
// distributed except according to those terms.
//! # Interner Adaptors
//!
//! Each type in this module provides some additional functionality beyond that
//! of Symtern's basic interner.
//!
//! Adaptors are wrapper types, i.e. they "wrap" another interner type, taking
//! it as a generic parameter.
//!
//! ```rust
//! use symtern::{Pool, Sym};
//! use symtern::adaptors::{Inline, InlineSym};
//!
//! type MyPool = symtern::adaptors::Inline<symtern::Pool<str,u32>>;
//! type MySym = symtern::adaptors::InlineSym<symtern::Sym<u32>>;
//! ```
//!
//! ## Inline
//!
//! By wrapping your `Pool<str, _>` type in the [`Inline`] adaptor, you can
//! create an interner optimized for short strings.  When input strings are
//! under a certain length, this adaptor will store them directly in the
//! returned symbols &mdash; entirely bypassing the wrapped interner.  If you
//! expect to be working with many short strings, it may perform better than
//! the basic interner.
//!
//! ```rust file="examples/combining-adaptors.rs" id="inline"
//! use symtern::prelude::*;
//! use symtern::adaptors::Inline;
//! use symtern::Pool;
//! let mut pool = Inline::from(Pool::<str,u64>::new());
//!
//! if let (Ok(hello), Ok(world)) = (pool.intern("Hello"), pool.intern("World")) {
//!     assert!(hello != world);
//!
//!     // Since both "hello" and "world" are smaller than the pool's
//!     // symbol representation (u64), neither symbol takes up any space
//!     // in the pool.
//!     assert!(pool.is_empty());
//! }
//! ```
//!
//! ## Luma
//!
//! The [`Luma`] adaptor uses interior mutability via `RefCell` to allow its
//! symbols to carry a lifetime parameter, which is used to prevent the pool
//! from being dropped while it is in use.
//!
//! For example, the following code will not compile because it attempts to
//! return a symbol from a temporary `Luma`-wrapped interner.
//! 
//! ```rust file="tests/compile-fail/luma-is-lifetime-safe.rs" id="example"
//! ```
//!
//! [`Luma`]: struct.Luma.html
//! [`Inline]: struct.Inline.html

mod inline;
mod luma;

pub use self::inline::{Inline, Sym as InlineSym};
pub use self::luma::{Luma, Sym as LumaSym};

#[cfg(test)]
mod tests {
    use Pool;
    use super::{Inline, Luma};

    #[test]
    fn can_inline_luma() {
        let inline: Inline<Luma<Pool<str, u64>>> = Inline::new();
        let x = inline.intern("x").expect("failed to inline a value");
        let y = inline.intern("y").expect("failed to inline a value");

        assert_eq!(Ok("x"), inline.resolve(&x));
        assert_eq!(Ok("y"), inline.resolve(&y));
    }

    #[test]
    fn can_luma_inline() {
        let luma: Luma<Inline<Pool<str, u64>>> = Luma::new();
        let x = luma.intern("x").expect("failed to inline a value");
        let y = luma.intern("y").expect("failed to inline a value");

        assert_eq!(Ok("x"), luma.resolve(&x));
        assert_eq!(Ok("y"), luma.resolve(&y));
    }
}
