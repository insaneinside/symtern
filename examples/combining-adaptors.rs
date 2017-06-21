// Copyright (C) 2016 Symtern Project Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-Apache
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. This file may not be copied, modified, or
// distributed except according to those terms.
//! Demonstration of combining adaptors.
extern crate symtern;

fn main() {
    #![allow(unused_variables, unused_mut)]

    /*` id=inline */ {
        use symtern::prelude::*;
        use symtern::adaptors::Inline;
        use symtern::Pool;

        let mut pool = Inline::from(Pool::<str,u64>::new());

        if let (Ok(hello), Ok(world)) = (pool.intern("Hello"), pool.intern("World")) {
            assert!(hello != world);

            // Since both "hello" and "world" are smaller than the pool's
            // symbol representation (u64), neither symbol takes up any space
            // in the pool.
            assert!(pool.is_empty());
        }            
    }


    /*/*` id=basic-luma-inline */ {
        let luma_inlined = Inliner::from(Luma::from(Pool::<str,u64>::new()));
        let sym = (&luma_inlined).intern("woot!");
    }*/
}
