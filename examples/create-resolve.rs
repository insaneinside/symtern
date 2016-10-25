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
        use symtern::traits::*;

        let mut basic_pool = symtern::basic::Pool::<str,u16>::new();
        let cat = basic_pool.intern("Kibbles").expect("Failed to intern the cat");

        let mut short_pool = symtern::short::Pool::<u64>::new();
        let dog = short_pool.intern("Fido").expect("Failed to intern the dog");
        //` }

        // With interners that implement the `Resolve` trait, we can resolve
        // a symbol back into its referent by passing the symbol to the interner's
        // `resolve` method by value.
        //` id=resolve {
        assert_eq!(Ok("Kibbles"), basic_pool.resolve(cat));
        //` }


        // Some interners -- for whatever implementation-specific reason -- require
        // a reference instead of a copy of a symbol in order to resolve it.
        // In these cases we use the `resolve_ref` function, which is so named to
        // avoid confusion over the different syntax we use when calling it.
        //` id=resolve_ref {
        assert_eq!(Ok("Fido"), short_pool.resolve_ref(&dog));
        //` }
    }

    intern_with_error_handling().expect("we didn't actually expect an error here!");
    resolve_with_error_handling().expect("we didn't actually expect an error here!");
    resolve_ref_with_error_handling().expect("we didn't actually expect an error here!");
}

//` ignore {

// For these examples we create some "fake" context around the example so we
// can demonstrate what error handling might look like when using custom
// error types.
type MyErrorType = symtern::Error;
use symtern::traits::*;
use symtern::{basic, short};
use symtern::Result;

#[allow(unused_variables)]
fn intern_with_error_handling() -> Result<()> {
    let mut some_interner = short::Pool::<u64>::new();
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
    let mut some_pool = basic::Pool::<str,u8>::new();
    let sym = some_pool.intern("abc").unwrap();

    //` id=resolve-with-error-handling {
    let s = match some_pool.resolve(sym) {
        Ok(s) => s,
        Err(err) => return Err(MyErrorType::from(err)),
    };
    //` }
    Ok(())
}

#[allow(unused_variables)]
fn resolve_ref_with_error_handling() -> Result<()> {
    let mut some_pool = short::Pool::<u32>::new();
    let sym = some_pool.intern("abc").unwrap();

    //` id=resolve_ref-with-error-handling {
    let s = match some_pool.resolve_ref(&sym) {
        Ok(s) => s,
        Err(err) => return Err(MyErrorType::from(err)),
    };
    //` }
    Ok(())
}
//` }
