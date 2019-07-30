// Copyright (C) 2016 Symtern Project Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-Apache
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. This file may not be copied, modified, or
// distributed except according to those terms.
//! Demonstration of combining adaptors.
extern crate symtern;
use symtern::prelude::*;
use symtern::Pool;
use symtern::adaptors::Inline;

fn main() {
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

    /*` id=basic-inline-luma { */ {
        // Once we've constructed an adaptor from an existing pool, we _cannot_
        // resolve previously-created symbols!
        let mut basic_pool = Pool::<str,u64>::new();
        let _some_sym = basic_pool.intern("Mornin'!").expect("interning failed");

        let mut inline_pool = Inline::from(basic_pool);
        let _inline_sym = inline_pool.intern("G'day").expect("interning failed");

    /*    let luma_pool = Luma::from(inline_pool);
        let luma_sym = luma_pool.intern("Why, hello there!").expect("interning failed");*/
    }

    /*/*` id=basic-luma-inline { */ {
        let luma_inlined = Inliner::from(Luma::from(Pool::<str,u64>::new()));
        let sym = (&luma_inlined).intern("woot!");
    }*/
}
