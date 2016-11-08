// Copyright (C) 2016 Symtern Project Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-Apache
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. This file may not be copied, modified, or
// distributed except according to those terms.
//! Example for lib.rs to demonstrate that symbol types implement Copy.
extern crate symtern;
use symtern::prelude::*;
use symtern::Pool;

/// Take ownership of a value, consuming it.
fn consume<T>(_: T) {}

fn main() {
    let mut pool = Pool::<str, u32>::new();
    let sym = pool.intern("xyz").unwrap();
    consume(sym);
    println!("The symbol is still valid: {:?}", pool.resolve(sym));
}
