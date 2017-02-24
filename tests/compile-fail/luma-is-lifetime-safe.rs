// Copyright (C) 2016-2017 Symtern Project Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-Apache
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. This file may not be copied, modified, or
// distributed except according to those terms.
//! Test that the `Luma` adaptor does indeed prevent us from dropping an
//! interner before any of its symbols.

extern crate symtern;
//` id="example" {
use symtern::prelude::*;
use symtern::Pool as Basic;
use symtern::adaptors::Luma;

type Pool = Luma<Basic<str, u32>>;

/// Return a Sym from a temporary Luma-wrapped interner.  This causes a compile
/// error because the interner, which is dropped at the end of the function, is
/// referenced by the returned symbol.
fn make_sym<'a>(s: &str) -> <&'a Pool as symtern::traits::Intern>::Symbol {
    Pool::new().intern(s).unwrap() //~ ERROR borrowed value does not live long enough
}
//` }

//` ignore {
fn main() {
    let s = make_sym("he who smelt it, dealt it");
    println!("s = {:?}", s);
}
//` }
