// Copyright (C) 2016 Symtern Project Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-Apache
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. This file may not be copied, modified, or
// distributed except according to those terms.
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
