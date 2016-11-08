// Copyright (C) 2016 Symtern Project Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-Apache
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. This file may not be copied, modified, or
// distributed except according to those terms.
//! Simple example of using a symtern interner.
extern crate symtern;

fn main() {
    /*` id=basic */ {
        // Import Symtern's traits, which allow us to use each interner the same way
        // regardless of the underlying implementation.
        use symtern::prelude::*;

        // Create a new pool that accepts `&str` arguments to `intern`, and uses
        // `u8` as the backing representation for its symbol type.
        let mut pool = symtern::Pool::<str,u8>::new();
        if let (Ok(hello), Ok(world)) = (pool.intern("Hello"), pool.intern("World")) {
            assert!(hello != world);

            assert_eq!(hello, hello);
            assert_eq!(Ok(hello), pool.intern("Hello"));
            assert_eq!(Ok("Hello"), pool.resolve(hello));

            assert_eq!(world, world);
            assert_eq!(Ok(world), pool.intern("World"));
            assert_eq!(Ok("World"), pool.resolve(world));
        }
    }
}
