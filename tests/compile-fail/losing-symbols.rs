// Copyright (C) 2016 Symtern Project Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-Apache
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. This file may not be copied, modified, or
// distributed except according to those terms.
//! Erroneous example of trying to use unwrapped symbols with produced by
//! a pool that has since been wrapped in an adaptor.
extern crate symtern;
use symtern::prelude::*;
use symtern::Pool;
use symtern::adaptors::Inline;

fn main() {
    // Once we've constructed an adaptor from an existing pool, we _cannot_
    // resolve previously-created symbols:
    let mut basic_pool = Pool::<str,u64>::new();
    let some_sym = basic_pool.intern("Mornin'!").expect("interning failed");
    // After we've created `inline_pool`, consuming `basic_pool`...
    let mut inline_pool = Inline::from(basic_pool);
    // ...we won't be able to resolve `some_sym` because its type is
    // incompatible with the inline pool's `resolve` method!
    println!("{}", inline_pool.resolve(&some_sym).expect("resolution failed")); //~ ERROR mismatched types [E0308]
}
