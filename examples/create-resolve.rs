// Copyright (C) 2016 Symtern Project Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-Apache
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. This file may not be copied, modified, or
// distributed except according to those terms.
//! Examples intended for the documentation in src/traits.rs.  This code
//! demonstrates the basics of using Symtern for interning strings and
//! resolving the resulting symbols.
extern crate symtern;

fn main() {
    {
        // Symbol creation: just call `intern` on the symbol pool, passing it
        // a reference to the value to be interned.
        //` id=create {
        // Nearly all functionality is in the trait implementations, so we simply
        // glob-import the traits.
        use symtern::prelude::*;
        use symtern::Pool;
        use symtern::adaptors::Inline;


        let mut basic_pool = Pool::<str,u16>::new();
        let cat = basic_pool.intern("Kibbles").expect("Failed to intern the cat");

        let mut inline_pool = Inline::<Pool<str, u64>>::new();
        let dog = inline_pool.intern("Fido").expect("Failed to intern the dog");
        //` }

        // With interners that implement the `Resolve` trait, we can resolve
        // a symbol back into its referent by passing the symbol to the interner's
        // `resolve` method by value.
        //` id=resolve {
        assert_eq!(Ok("Kibbles"), basic_pool.resolve(cat));
        //` }


        // Some interners -- for whatever implementation-specific reason --
        // require a reference instead of a copy of a symbol in order to
        // resolve it; look for a reference type in the `Input` associated type
        // of type's `Resolve` implementation to identify them.l
        //` id=resolve_ref {
        assert_eq!(Ok("Fido"), inline_pool.resolve(&dog));
        //` }
    }

    intern_with_error_handling().expect("we didn't actually expect an error here!");
    resolve_with_error_handling().expect("we didn't actually expect an error here!");
    resolve_unchecked().expect("we didn't actually expect an error here!");
}

//` ignore {

// For these examples we create some "fake" context around the example so we
// can demonstrate what error handling might look like when using custom
// error types.
type MyErrorType = symtern::Error;
use symtern::prelude::*;
use symtern::adaptors::Inline;
use symtern::{Pool, Result};

#[allow(unused_variables)]
fn intern_with_error_handling() -> Result<()> {
    let mut some_interner = Inline::<Pool<str,u64>>::new();
    //` id=intern-with-error-handling {
    let symbol = match some_interner.intern("Rosebud") {
        Ok(sym) => sym,
        Err(err) => return Err(MyErrorType::from(err)),
    };
    //` }
    Ok(())
}

#[allow(unused_variables)]
fn resolve_with_error_handling() -> Result<()> {
    let mut some_pool = Pool::<str,u8>::new();
    let sym = some_pool.intern("abc").unwrap();

    //` id=resolve-with-error-handling {
    let s = match some_pool.resolve(sym) {
        Ok(s) => s,
        Err(err) => return Err(MyErrorType::from(err)),
    };
    //` }
    Ok(())
}

fn resolve_unchecked() -> Result<()> {
    //` id=resolve_unchecked {
    let mut pool = Pool::<str, u8>::new();
    let sym = try!(pool.intern("abc"));

    assert_eq!("abc", unsafe { pool.resolve_unchecked(sym) });
    //` }
    Ok(())
}
//` }
