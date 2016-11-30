// Copyright (C) 2016 Symtern Project Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-Apache
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. This file may not be copied, modified, or
// distributed except according to those terms.

//` ignore {
// error-pattern:already borrowed: BorrowMutError
//` }

//! Check that using the `Luma` adaptor turns the compile-time-error "cannot
//! borrow `pool` as mutable because it is also borrowed as immutable" into
//! a run-time error, as asserted in the adaptor's documentation.

extern crate symtern;

use symtern::prelude::*;
use symtern::Pool;
use symtern::adaptors::Luma;

fn main() {
    let mut pool = Luma::from(Pool::<str, u32>::new());
    let x = pool.intern("foo").expect("failed to intern a value");
    let foo = pool.resolve(x).expect("failed to resolve the value we just interned");

    let _ = pool.intern("bar").expect("failed to intern a value"); //~ PANIC already borrowed: BorrowMutError
}
