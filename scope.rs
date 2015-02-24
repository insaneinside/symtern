/**@file
 *
 * Module `symbol::scope`: traits useful for defining scope-like hierarchies of
 * value maps, keyed by Symbols.
 */
use super::Nameable;
use super::super::util::intrusive;

/** Trait that defines a method to find the root of a scope hierarchy.
 *
 * @tparam T Root scope type.
 */
pub trait HasRootScope<T>
{ fn root_scope(&self) -> intrusive::Ref<T>; }

#[macro_export]
macro_rules! default_has_root_scope_impl {
    ($target:ty, $rstype:ty, $($mbr:ident).*) => (
        impl HasRootScope<$rstype> for $target {
            fn root_scope(&self) -> intrusive::Ref<$rstype> {
                self . $($mbr).+ . root_scope()
            }
        }
    );
}


/** Trait for types that define a method to find an instance's parent scope.
 */
pub trait HasParentScope<T>
{ fn parent_scope(&self) -> intrusive::Ref<T>; }

#[macro_export]
macro_rules! default_has_parent_scope_impl {
    ($target:ty, $pstype:ty, $($mbr:ident).*) => (
        impl HasParentScope<$pstype> for $target {
            fn parent_scope(&self) -> intrusive::Ref<$pstype> {
                self . $($mbr).+ . clone()
            }
        }
    );
}

pub trait RootScope<ValueType>: HasRootScope<Self> where ValueType: super::Nameable
{}

/*

/* ****************************************************************
 * SymbolTable
 */

/** A flexible, generic symbol-table utility for generated analyzers.
 */
pub struct
SymbolTable<RootScopeType: intrusive::RefCounted, 
            ParentScopeType: HasRootScope<RootScopeType> + intrusive::RefCounted, 
            ValueType: super::Nameable + HasParentScope<ParentScopeType> + intrusive::RefCounted + std::fmt::Debug>
{
    name: intrusive::Ref<Symbol>,
    search_upward: bool,
    search_downward: bool,
    max_search_depth: usize,
    pool: Rc<SymbolPool>,
    map: HashMap<intrusive::Ref<Symbol>,intrusive::Ref<ValueType>>,
    root_scope: intrusive::Ref<RootScopeType>,
    parent_scope: intrusive::Ref<ParentScopeType>
}

/* ================================================================
 * impl Eq, PartialEq
 */
impl<RootScopeType: intrusive::RefCounted + Eq,
     ParentScopeType: HasRootScope<RootScopeType> + intrusive::RefCounted + Eq,
     ValueType: super::Nameable + HasParentScope<ParentScopeType> + intrusive::RefCounted + std::fmt::Debug>
Eq for SymbolTable<RootScopeType,ParentScopeType,ValueType> {}

impl<RootScopeType: intrusive::RefCounted + Eq,
     ParentScopeType: HasRootScope<RootScopeType> + intrusive::RefCounted + Eq,
     ValueType: super::Nameable + HasParentScope<ParentScopeType> + intrusive::RefCounted + std::fmt::Debug>
PartialEq<SymbolTable<RootScopeType,ParentScopeType,ValueType>> 
for SymbolTable<RootScopeType,ParentScopeType,ValueType> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.search_upward == other.search_upward && 
        self.search_downward == other.search_downward && self.max_search_depth == other.max_search_depth && 
        self.pool == other.pool && self.map == other.map && self.root_scope == other.root_scope && 
        self.parent_scope == other.parent_scope
    }
}


/* ================================================================
 * impl std::fmt::Debug
 */
impl<RootScopeType: intrusive::RefCounted,
     ParentScopeType: HasRootScope<RootScopeType> + intrusive::RefCounted,
     ValueType: super::Nameable + HasParentScope<ParentScopeType> + intrusive::RefCounted + std::fmt::Debug>
std::fmt::Debug
for SymbolTable<RootScopeType,ParentScopeType,ValueType>
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut first = true;
        try!(write!(f, "{{"));

        for (key, value) in self.map.iter() {
            if first {
                try!(write!(f, ", "));
                first = false;
            }
            try!(write!(f, "{:?}: {:?}", key, *value));
        }
        write!(f, "}}")
    }
}

/* ================================================================
 * impl Nameable
 */
impl<RootScopeType: intrusive::RefCounted,
     ParentScopeType: HasRootScope<RootScopeType> + intrusive::RefCounted,
     ValueType: super::Nameable + HasParentScope<ParentScopeType> + intrusive::RefCounted + std::fmt::Debug>
Nameable
for SymbolTable<RootScopeType,ParentScopeType,ValueType>
{
    fn name(&self) -> intrusive::Ref<Symbol>
    { self.name.clone() }
}


/* ================================================================
 * impl HasParentScope<ParentScopeType>
 */
impl<RootScopeType: intrusive::RefCounted,
     ParentScopeType: HasRootScope<RootScopeType> + intrusive::RefCounted,
     ValueType: super::Nameable + HasParentScope<ParentScopeType> + intrusive::RefCounted + std::fmt::Debug>
HasParentScope<ParentScopeType>
for SymbolTable<RootScopeType,ParentScopeType,ValueType>
{
    fn parent_scope(&self) -> intrusive::Ref<ParentScopeType>
    { self.parent_scope.clone() }
}


/* ================================================================
 * impl HasRootScope<ParentScopeType>
 */
impl<RootScopeType: intrusive::RefCounted,
     ParentScopeType: HasRootScope<RootScopeType> + intrusive::RefCounted,
     ValueType: super::Nameable + HasParentScope<ParentScopeType> + intrusive::RefCounted + std::fmt::Debug>
HasRootScope<RootScopeType>
for SymbolTable<RootScopeType,ParentScopeType,ValueType>
{
    fn root_scope(&self) -> intrusive::Ref<RootScopeType> {
        self.root_scope.clone()
    }
}
*/
