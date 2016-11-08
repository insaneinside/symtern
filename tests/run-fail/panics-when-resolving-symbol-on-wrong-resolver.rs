// Copyright (C) 2016 Symtern Project Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-Apache
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. This file may not be copied, modified, or
// distributed except according to those terms.
// error-pattern:Detected an invalid attempt to resolve a symbol
#![cfg(debug_assertions)]
extern crate symtern;
use symtern::prelude::*;
use symtern::Pool;

fn main() {
    let mut p1 = Pool::<str,u16>::new();
    let mut p2 = Pool::<str,u16>::new();

    let s1 = p1.intern("foo").unwrap();
    let s2 = p2.intern("bar").unwrap();

    p1.resolve(s2).unwrap();
}
