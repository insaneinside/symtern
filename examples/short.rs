//! Basic example of using Symtern's `short` symbol pool, which encodes strings
//! directly in the returned symbols whenever possible.
extern crate symtern;
use symtern::traits::*;

fn main() {
    let mut pool = symtern::short::Pool::<u64>::new();
    let hello = pool.intern("Hello").expect("failed to intern a value");
    let world = pool.intern("World").expect("failed to intern a value");

    assert!(hello != world);

    assert_eq!((Ok("Hello"), Ok("World")),
               (pool.resolve_ref(&hello),
                pool.resolve_ref(&world)));

    // Since both "Hello" and "World" are short enough to be inlined, they
    // don't take up any space in the pool.
    assert_eq!(0, pool.len());
}
