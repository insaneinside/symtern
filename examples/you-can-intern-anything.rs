// Copyright (C) 2016 Symtern Project Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-Apache
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. This file may not be copied, modified, or
// distributed except according to those terms.
//! Symtern's `basic` interner is generic over the interned object type -- in
//! other words, you can intern *anything* that implements `Eq`, `Hash`, and
//! `ToOwned`.  Let's try it!
extern crate symtern;
use symtern::prelude::*;
use symtern::Pool;

#[derive(Clone, Eq, PartialEq, Hash)]
struct WibbleWobble {
    whee: Vec<u32>
}

fn main() {
    let mut pool = Pool::<_,u8>::new();
    assert!(pool.intern(&WibbleWobble{whee: vec![1, 2, 3, 4, 5]}).is_ok());
}
