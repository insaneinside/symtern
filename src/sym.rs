// Copyright (C) 2016-2017 Symtern Project Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-Apache
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. This file may not be copied, modified, or
// distributed except according to those terms.
//! Internal helpers for creating and manipulating interned-value standins
//! (symbols).
//!
//! The traits defined in this module should be used **only** when you are
//! implementing your own interner or adaptor types.  Because they allow you to
//! create symbols out of thin air and inspect implementation details, Bad
//! Things™ are likely to happen if you use their methods in other contexts.
use std::hash::Hash;
use ::num_traits::{Bounded, Unsigned, FromPrimitive, ToPrimitive};

use traits;

/// Trait describing primitive types used as symbols' internal representations.
pub trait SymbolId: Copy + Eq + Hash + Bounded + Unsigned + FromPrimitive + ToPrimitive {}
impl<T> SymbolId for T where T: Copy + Eq + Hash + Bounded + Unsigned + FromPrimitive + ToPrimitive {}
/// Type that will be used for `Pool::Id` in all generated `Pool` impls.
pub type PoolId = usize;


/// Internal trait for Pool types that provides a consistent symbol-creation
/// interface regardless of whether or not the crate is compiled in debug mode.
pub trait Pool {
    /// Symbol type associated with the pool; this should be the same as the
    /// associated type of the same name in any `Interner` implementations.
    type Symbol: Symbol;

    /// Fetch the pool's ID.
    #[cfg(debug_assertions)]
    fn id(&self) -> PoolId;

    /// Create a symbol with the specified ID.  Do **not** use this method
    /// unless you are implementing a new symbol pool or adaptor type!
    /// Any created symbol _must_ be resolvable on an existing pool.
    fn create_symbol(&self, id: <Self::Symbol as Symbol>::Id) -> Self::Symbol;
}

/// Interface used to extract internal ID values from symbols.
pub trait Symbol: traits::Symbol {
    /// Primitive type underlying the symbol implementation.
    type Id: SymbolId;

    /// Fetch the ID of the pool to which the symbol belongs.
    #[cfg(debug_assertions)]
    fn pool_id(&self) -> PoolId;

    /// Fetch the symbol's ID by value.
    fn id(&self) -> Self::Id;

    /// Fetch a reference to the symbol's ID.
    fn id_ref(&self) -> &Self::Id;

    /// Fetch the symbol's ID by value.
    fn id(&self) -> Self::Id {
        *self.id_ref()
    }
}

/// Interface for creating new symbols from raw IDs.
pub trait Create: Symbol {
    /// Create a new symbol with the given ID and source pool.
    #[cfg(debug_assertions)]
    fn create(id: Self::Id, pool_id: PoolId) -> Self;

    /// Create a new symbol with the given ID.
    #[cfg(not(debug_assertions))]
    fn create(id: Self::Id) -> Self;
}



