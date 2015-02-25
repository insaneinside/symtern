///! Defines the `Nameable` trait, for objects with names.
use super::Symbol;
use std::fmt;

/// Trait for use with objects named by symbols 
pub trait Nameable {
    fn name(&self) -> Symbol;
}

impl<'t> fmt::Display for Nameable + 't {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[macro_export]
macro_rules! default_nameable_impl {
    ($target:ty) => (
        impl symbol::Nameable for $target {
            fn name(&self) -> Symbol {
                self.name
            }
        });

    ($target:ty, $($mbr:ident).+) => (
        impl symbol::Nameable for $target {
            fn name(&self) -> Symbol {
                self.$($mbr).+
            }
        });
}

