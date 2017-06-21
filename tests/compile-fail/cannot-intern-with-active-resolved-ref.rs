// Copyright (C) 2016 Symtern Project Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-Apache
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. This file may not be copied, modified, or
// distributed except according to those terms.
//! Check that we're unable to intern new values while holding a reference
//! obtained from `resolve`.
//!
//! This is used as an example in src/adaptors/luma.rs.
extern crate symtern;
use symtern::prelude::*;
use symtern::Pool;

fn main() {
    let mut pool = Pool::<str, u32>::new();
    let x = pool.intern("foo").expect("failed to intern a value");
    let foo = pool.resolve(x).expect("failed to resolve the value we just interned");

    let _ = pool.intern("bar").expect("failed to intern a value"); //~ ERROR cannot borrow `pool` as mutable because it is also borrowed as immutable
}
