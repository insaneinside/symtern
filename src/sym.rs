//! Internal helpers for creating and manipulating interned-value standins
//! (symbols).

use traits::SymbolId;

/// Interface presented by symbol types created with the
/// [`make_sym`](macro.make_sym.html) macro.
pub trait Symbol {
    /// Primitive type underlying the symbol implementation.
    type Id: SymbolId;
    /// Fetch the symbol's ID by value.
    fn id(&self) -> Self::Id;
    /// Fetch a reference to the symbol's ID.
    fn id_ref(&self) -> &Self::Id;
    /// Create a new value with the given ID.
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
    (@impl $name:ident < $I: ident > ( $wrapped: path ) ; $($bound: tt)+) => {
        impl<$I> ::sym::Symbol for $name<$I>
            where $I: $($bound)+
        {
            type Id = $I;
            fn id(&self) -> Self::Id { self.wrapped.id() }
            fn id_ref(&self) -> &Self::Id { self.wrapped.id_ref() }
            fn create(id: Self::Id) -> Self {
                $name{wrapped: <$wrapped as ::sym::Symbol>::create(id)}
            }
        }
    };

    // @impl for unwrapped symbol types
    (@impl $name:ident < $I: ident >; $($bound: tt)+) => {
        impl<$I> ::sym::Symbol for $name<$I>
            where $I: $($bound)+
        {
            type Id = $I;
            fn id(&self) -> Self::Id { self.id }
            fn id_ref(&self) -> &Self::Id { &self.id }
            fn create(id: Self::Id) -> Self {
                $name{id: id}
            }
        }
    };

    // @struct for wrapped symbol types
    (@struct $name:ident < $I: ident > ( $wrapped: path ): $doc:expr; $($bound: tt)+) => {
        #[doc = $doc]
        #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
        pub struct $name<$I: $($bound)+> {
            wrapped: $wrapped,
        }
    };
    // @impl for unwrapped symbol types
    (@struct $name:ident < $I: ident >: $doc:expr; $($bound: tt)+) => {
        #[doc = $doc]
        #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
        pub struct $name<$I: $($bound)+> {
            id: $I,
        }
    };
    // Entry point for unwrapped symbol types
    (pub $name:ident < $I:ident $(: $bound: ident $(+ $rbound: ident)*)* >: $doc: expr; $($rest: tt)*)
        => { make_sym!(@struct $name<$I>: $doc; SymbolId + $($bound $( + $rbound)*)*);
             make_sym!(@impl $name<$I>; SymbolId + $($bound $( + $rbound)*)*);
             make_sym!($($rest)*); };

    // Entry point for wrapped symbol types
    (pub $name: ident < $I:ident $(: $bound: ident $(+ $rbound: ident)*)* >($wrapped: path): $doc: expr; $($rest: tt)*)
        => { make_sym!(@struct $name<$I>($wrapped): $doc; SymbolId + $($bound $( + $rbound)*)*);
             make_sym!(@impl $name<$I>($wrapped); SymbolId + $($bound $( + $rbound)*)*);
             make_sym!($($rest)*); };
}
