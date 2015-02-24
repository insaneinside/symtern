///! Symbol- and name-storage facilities. 
use std;
use std::rc::Rc;
use std::hash::{hash, SipHasher};
use std::hash::{Hash, Hasher};
use std::collections::HashSet;

use util::intrusive;
use util::intrusive::{Ref,RefCounted};


/// Trait for use with objects named by symbols 
pub trait Nameable {
    fn name(&self) -> Ref<Symbol>;
}

#[macro_export]
macro_rules! default_nameable_impl {
    ($target:ty) => (
        impl symbol::Nameable for $target {
            fn name(&self) -> Ref<Symbol> {
                self.name.clone()
            }
        });

    ($target:ty, $($mbr:ident).+) => (
        impl symbol::Nameable for $target {
            fn name(&self) -> Ref<Symbol> {
                (self.$($mbr).+).clone()
            }
        });
}


use std::fmt;

impl<'t> fmt::Display for Nameable + 't {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

// #[macro_use]
// pub mod scope;


/// A unique set of names. 
pub type SymbolPool = HashSet<Symbol>;

/* ****************************************************************
 * Symbol
 */
type HashType = SipHasher;
pub type ID = u64;


/// An atomic name; an entry in a `SymbolTable` and a `SymbolPool`.
#[derive(Debug)]
pub struct
Symbol
{
    pub id: ID,
    pub name: Rc<String>,               /*< Symbol name */
    refcount: usize
}

impl Symbol
{
    /// Create a new Symbol instance.
    /// 
    /// @param name Symbol name.
    pub fn new(name: &str) -> Symbol {
        let hsh = Symbol::hash(&name);
        Symbol{id: hsh, name: Rc::new(name.to_string()), refcount: 1}
    }

    // pub fn new_on_heap(name: &str) -> Ref<Symbol> {
    //     let hsh = Symbol::hash(&name);
    //     intrusive::ref_to_new(Symbol{id: hsh, name: Rc::new(name.to_string()), refcount: 0})
    // }

    pub fn hash(name: &str) -> u64
    {
        hash::<_, HashType>(&name)
    }
}


impl intrusive::ExplicitlySized for Symbol {
    fn get_type_size(&self) -> usize { std::mem::size_of::<Self>() }
    fn get_type_align(&self) -> usize { std::mem::align_of::<Self>() }
}


impl Nameable for Symbol {
    fn name(&self) -> Ref<Symbol> { <Self as intrusive::RefCounted>::ref_to_self() }
}

impl Hash for Symbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}


impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Eq for Symbol {}
impl PartialEq<Symbol> for Symbol
{
    fn eq(&self, other: &Symbol) -> bool
    {
        self.id == other.id
    }
}

default_refcounted_impl!(Symbol, refcount);
