//! Examples of errors and error handling when working with Symtern.
//!
//! Keep this file in sync with the error-handling example in README.md.
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

    //` id=no-such-symbol
    {
        let mut p1 = Pool::<str,u32>::new();
        let p2 = Pool::<str,u32>::new();

        match p2.resolve(p1.intern("Batman").unwrap()) {
            Ok(s) => panic!("Expected NoSuchSymbol, but got string {:?}", s),
            Err(err) => match err.kind() {
                ErrorKind::NoSuchSymbol => (),
                _ => panic!("Wrong error returned from `resolve`: {}", err),
            }
        }
    }
}
