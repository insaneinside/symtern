//! Simple example of using a symtern interner.
//!
//! Keep this file in sync with the initial example in README.md.
extern crate symtern;

use std::mem;

// Import Symtern's traits, which allow us to use each interner the same way
// regardless of the underlying implementation.
use symtern::traits::*;

// We'll use the "basic" interner, which is generic over both the interned
// value-type and the primitive type used to represent symbols.
use symtern::basic::Pool;

fn main() {
    // Create a new pool that accepts `&str` arguments to `intern`.
    let mut pool = Pool::<str,u8>::new();
    if let (Ok(hello), Ok(world)) = (pool.intern("Hello"), pool.intern("World")) {

        // Symbols are as small as specified.
        assert_eq!(mem::size_of_val(&hello), mem::size_of::<u8>());

        assert!(hello != world);

        assert_eq!(hello, hello);
        assert_eq!(Ok(hello), pool.intern("Hello"));
        assert_eq!(Ok("Hello"), pool.resolve(hello));

        assert_eq!(world, world);
        assert_eq!(Ok(world), pool.intern("World"));
        assert_eq!(Ok("World"), pool.resolve(world));
    }
}
