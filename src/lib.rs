//! # Selection of interning facilities in Rust
//!
//! Symtern provides interning facilities for several use-cases (because let's
//! face it: it's one of the least exciting parts of writing a parser).
//!
//!   * [`basic`] is generic over the type of interned values, and can be
//!     configured to use any of Rust's numeric primitives for symbol IDs.
//!     It is the recommended interner for most purposes.
//!
//!   * [`short`] is optimized for short strings, which are stored directly in
//!     the returned symbol value when under a certain length.
//!
//!
//! Each of these modules defines a `Pool` type that implements
//! [`traits::Interner`].
//!
//! # Examples
//!
//! [Symbol types](traits/trait.Symbol.html) are `Copy`:  they can be passed by
//! value without resulting in a move.
//!
//! ```rust
//! extern crate symtern;
//! use symtern::basic::Pool;
//! use symtern::traits::*;
//!
//! /// Take ownership of a value, consuming it.
//! fn consume<T>(v: T) {}
//!
//! fn main() {
//!     let mut pool = Pool::<str, u32>::new();
//!     let sym = pool.intern("xyz").unwrap();
//!     consume(sym);
//!     println!("The symbol is still valid: {:?}", pool.resolve(sym));
//! }
//! ```
//!
//! [`traits::Interner`]: traits/trait.Interner.html
//! [`basic`]: basic/index.html
//! [`short`]: short/index.html

extern crate num_traits;
#[cfg(feature = "fnv")] extern crate fnv;

use std::fmt;

#[macro_use] mod sym;
mod core;
pub mod traits;
pub mod short;
pub mod basic;

/// Result type used by fallible operations in symtern.
type Result<T> = std::result::Result<T, Error>;

/// Error type used by this crate.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Error {
    kind: ErrorKind,
}

impl Error {
    /// Create a new error with the given kind.
    pub fn new(kind: ErrorKind) -> Self {
        Error{kind: kind}
    }

    /// Get the kind of error this object represents.
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Error{kind: kind}
    }
}


impl std::error::Error for Error {
    fn description(&self) -> &str {
        match self.kind {
            ErrorKind::PoolOverflow => "out of space for new symbols",
            ErrorKind::NoSuchSymbol => "no such symbol found",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", <Self as std::error::Error>::description(self))
    }
}

/// Kinds of errors representable by the Error type.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    /// The underlying type used to uniquely identify symbols cannot represent
    /// any more values.
    PoolOverflow,
    
    /// The given symbol does not exist in the pool that was asked to
    /// resolve it.
    NoSuchSymbol,
}
