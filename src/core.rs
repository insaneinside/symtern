use std::hash::{Hash, Hasher};

#[cfg(feature = "fnv")]
pub type DefaultHashAlgo = ::fnv::FnvHasher;
#[cfg(not(feature = "fnv"))]
pub type DefaultHashAlgo = ::std::collections::hash_map::DefaultHasher;


/// Hash an object using the given hasher type.
pub fn hash<T: ?Sized + Hash, H: Hasher + Default>(obj: &T) -> u64 {
    let mut hasher = H::default();
    obj.hash(&mut hasher);
    hasher.finish()
}
