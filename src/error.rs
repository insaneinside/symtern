//! Error 

use std::fmt;

/// Result type used by fallible operations in symtern.
pub type Result<T> = ::std::result::Result<T, Error>;

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


impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        match self.kind {
            ErrorKind::PoolOverflow => "out of space for new symbols",
            ErrorKind::NoSuchSymbol => "no such symbol found",
            ErrorKind::__DoNotMatchThisVariant(_) => unreachable!(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", <Self as ::std::error::Error>::description(self))
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

    /// This enum is subject to change as additional interner implementations
    /// are added, so you should use an ident/wildcard to catch any variants
    /// you do not explicitly handle.
    #[doc(hidden)]
    __DoNotMatchThisVariant(())
}
