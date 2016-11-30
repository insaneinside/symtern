// Copyright (C) 2016 Symtern Project Contributors
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

use traits::{self, SymbolId};

/// Type that will be used for `Pool::Id` in all generated `Pool` impls.
pub type PoolId = usize;

/// Types used by interner implementations.
pub trait Types {
    /// Symbol type associated with the pool; this should be the same as the
    /// associated type of the same name in any `Interner` implementations.
    type Symbol: Symbol;

    /// Input accepted by reference as an argument to `intern` on `Interner` or
    /// `InternerMut`.
    type Input: ?Sized;

    /// Value returned by reference in the result from `resolve` on `Resolve`,
    /// `resolve_ref` on `ResolveRef`, or `resolve_unchecked` on
    /// `ResolveUnchecked`.
    type Output: ?Sized;
}

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
    fn create_symbol(self, id: <Self::Symbol as Symbol>::Id) -> Self::Symbol;
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

    /// Create a new value with the given ID and source pool.
    #[cfg(debug_assertions)]
    fn create(id: Self::Id, pool_id: PoolId) -> Self;

    /// Create a new symbol with the given ID.
    #[cfg(not(debug_assertions))]
    fn create(id: Self::Id) -> Self;
}

impl<'a, T> Types for &'a T
    where T: Types
{
    type Symbol = T::Symbol;
    type Input = T::Input;
    type Output = T::Output;
}

impl<'a, T> Types for &'a mut T
    where T: Types
{
    type Symbol = T::Symbol;
    type Input = T::Input;
    type Output = T::Output;
}

/// Define an opaque type constructor wrapping an underlying primitive ID, or
/// other symbol type, to be used as a symbol type.  When wrapping a primitive
/// ID type, the mandatory type parameter is automatically bounded by
/// [`traits::SymbolId`], and its instance is available via the private
/// `id` field.
///
/// Basic usage (wrapping primitive ID types):
///
/// ```rust,ignore
/// make_sym! {
///     pub MySym<I>: "My very own symbol type with its very own doc-string";
///     pub AnotherSym<J: ExtraTraitBound>: "This one has an extra trait bound on the primitive ID type."
/// }
/// ```
///
/// To wrap another symbol type, place it in parentheses after the
/// generic-parameters list.  The wrapped value will be placed in a a private
/// field `wrapped`.
///
/// ```rust,ignore
/// make_sym! {
///     pub WrapperSym<W>(W): "Wraps `W` for extra hugs.";
/// }
/// ```
macro_rules! make_sym {
    () => {};

    // @impl for wrapped symbol types
    (@impl $name:ident < $I: ident > ( $wrapped: path ) ; $($bound: tt)+ ) => {
        impl<$I> ::sym::Symbol for $name<$I>
            where $I: $($bound)+,
                  $wrapped: ::sym::Symbol
        {
            type Id = <$wrapped as ::sym::Symbol>::Id;

            #[cfg(debug_assertions)]
            fn pool_id(&self) -> ::sym::PoolId {
                self.wrapped.pool_id()
            }

            fn id(&self) -> Self::Id { self.wrapped.id() }
            fn id_ref(&self) -> &Self::Id { self.wrapped.id_ref() }

            #[cfg(not(debug_assertions))]
            fn create(id: Self::Id) -> Self {
                $name{wrapped: <$wrapped as ::sym::Symbol>::create(id)}
            }

            #[cfg(debug_assertions)]
            fn create(id: Self::Id, pool_id: ::sym::PoolId) -> Self {
                $name{wrapped: <$wrapped as ::sym::Symbol>::create(id, pool_id)}
            }
        }

        impl<$I> From<$wrapped> for $name<$I>
            where $I: $($bound)+
        {
            fn from(wrapped: $wrapped) -> Self {
                $name{wrapped: wrapped}
            }
        }
    };

    // @impl for unwrapped symbol types
    (@impl $name:ident < $I: ident > ; $($bound: tt)+) => {

        impl<$I> ::sym::Symbol for $name<$I>
            where $I: $($bound)+
        {
            type Id = $I;

            #[cfg(debug_assertions)]
            fn pool_id(&self) -> ::sym::PoolId {
                self.pool_id
            }

            fn id(&self) -> Self::Id { self.id }
            fn id_ref(&self) -> &Self::Id { &self.id }
            #[cfg(not(debug_assertions))]
            fn create(id: Self::Id) -> Self {
                $name{id: id}
            }
            #[cfg(debug_assertions)]
            fn create(id: Self::Id, pool_id: ::sym::PoolId) -> Self {
                $name{id: id, pool_id: pool_id}
            }
        }
    };

    // @struct for wrapped symbol types
    (@struct $name:ident < $I: ident > ( $wrapped: path ) : $doc:expr ; $($bound: tt)+) => {
        #[doc = $doc]
        #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
        pub struct $name<$I: $($bound)+ > {
            wrapped: $wrapped,
        }
    };

    // @impl for unwrapped symbol types
    (@struct $name:ident < $I: ident > : $doc:expr; $($bound: tt)+) => {
        #[doc = $doc]
        #[cfg(not(debug_assertions))]
        #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
        pub struct $name<$I: $($bound)+> {
            id: $I,
        }
        #[doc = $doc]
        #[cfg(debug_assertions)]
        #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
        pub struct $name<$I: $($bound)+> {
            id: $I,
            pool_id: ::sym::PoolId,
        }
    };

    // Entry point for unwrapped symbol types
    ($(#[$attr: meta])*
     pub $name:ident < $I:ident $(: $bound: ident $(+ $rbound: ident)*)* > : $doc: expr; $($rest: tt)*)
        => {$(#[$attr])*
            make_sym!(@struct $name<$I> : $doc; SymbolId $(+ $bound $( + $rbound)*)*);
            $(#[$attr])*
            make_sym!(@impl $name<$I> ; SymbolId $(+ $bound $( + $rbound)*)*);
            make_sym!($($rest)*); };

    // Entry point for wrapped symbol types
    ($(#[$attr: meta])*
     pub $name: ident < $I:ident $(: $bound: ident $(+ $rbound: ident)*)* >($wrapped: path) : $doc: expr; $($rest: tt)*)
        => {$(#[$attr])*
            make_sym!(@struct $name<$I>($wrapped) : $doc; SymbolId $(+ $bound $( + $rbound)*)*);
            $(#[$attr])*
            make_sym!(@impl $name<$I>($wrapped) ; SymbolId $(+ $bound $( + $rbound)*)*);
            make_sym!($($rest)*); };
}
