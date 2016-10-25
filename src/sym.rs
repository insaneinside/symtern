//! Internal helpers for creating and manipulating interned-value standins
//! (symbols).

use traits::{self, SymbolId};

/// Type that will be used for `Pool::Id` in all generated `Pool` impls.
pub type PoolId = usize;


/// Internal trait for Pool types that provides a consistent symbol-creation
/// interface regardless of whether or not the crate is compiled in debug mode.
pub trait Pool {
    /// Fetch the pool's ID.
    #[cfg(debug_assertions)]
    fn id(&self) -> PoolId;

    /// Symbol type associated with the pool; this should be the same as the
    /// associated type of the same name in its `Interner` or
    /// `InternerMut` implementation.
    type Symbol: Symbol;

    /// Create a symbol with the specified symbol ID.
    fn create_symbol(&self, id: <Self::Symbol as Symbol>::Id) -> Self::Symbol;
}

/// Interface presented by symbol types created with the
/// [`make_sym`](macro.make_sym.html) macro.
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

    /// Create a new value with the given ID.
    #[cfg(not(debug_assertions))]
    fn create(id: Self::Id) -> Self;
}

/// Define an opaque type constructor wrapping an underlying primitive ID, or
/// other symbol type, to be used as a symbol type.  The mandatory type
/// parameter is automatically bounded by [`traits::SymbolId`], and its
/// instance is available via the private `id` field.
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
///     pub WrapperSym<I>(MySym<I>): "Wraps MySym<I> for extra hugs.";
/// }
/// ```
macro_rules! make_sym {
    () => {};

    // @impl for wrapped symbol types
    (@impl $name:ident < $I: ident > ( $wrapped: path ) ; $($bound: tt)+ ) => {
        impl<$I> ::sym::Symbol for $name<$I>
            where $I: $($bound)+
        {
            type Id = $I;

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
