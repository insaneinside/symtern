// Copyright (C) 2016 Symtern Project Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-Apache
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. This file may not be copied, modified, or
// distributed except according to those terms.
//! Basic example of using Symtern's `Inline` symbol-pool adaptor, which
//! encodes strings directly in the returned symbols whenever possible.
extern crate symtern;
use symtern::prelude::*;
use symtern::Pool;
use symtern::adaptors::Inline;

fn main() {
    let mut pool = Inline::<Pool<str,u64>>::new();
    let hello = pool.intern("Hello").expect("failed to intern a value");
    let world = pool.intern("World").expect("failed to intern a value");

    assert!(hello != world);

    assert_eq!((Ok("Hello"), Ok("World")),
               (pool.resolve(&hello),
                pool.resolve(&world)));

    // Since both "Hello" and "World" are short enough to be inlined, they
    // don't take up any space in the pool.
    assert_eq!(0, pool.len());
}
