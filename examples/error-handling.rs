// Copyright (C) 2016 Symtern Project Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-Apache
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. This file may not be copied, modified, or
// distributed except according to those terms.
//! Examples of errors and error handling when working with Symtern.
extern crate symtern;

use symtern::traits::*;
use symtern::basic::Pool;
use symtern::ErrorKind;

fn main() {
    /*` id=overflow */ {
        // Here we create a pool that uses `u8` as the backing representation for its
        // symbol type, then proceed to completely fill it.
        let mut pool = Pool::<u16,u8>::new();
        for i in 0u16..256 {
            assert!(pool.intern(&i).is_ok(), "Failed to intern a value");
        }
        assert!(pool.is_full());

        // Any attempt to intern a previously-interned input should still work...
        assert!(pool.intern(&123).is_ok());
        // ...but new values will elicit a `PoolOverflow` error:
        match pool.intern(&1234) {
            Ok(sym) => panic!("Expected overflow, but got symbol {:?}", sym),
            Err(err) => match err.kind() {
                ErrorKind::PoolOverflow => (),
                _ => panic!("Wrong error kind returned from `intern`"),
            }
        }
    }
}
