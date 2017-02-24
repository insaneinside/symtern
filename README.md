# Symtern: Fast general-purpose interners for every use case
[![Build Status](https://travis-ci.org/insaneinside/symtern.svg?branch=master)](https://travis-ci.org/insaneinside/symtern)

Symtern is a selection of high-performance interner implementations written
in Rust.

Interners, which take complex values and map them to trivially-comparable
stand-ins that can later be resolved back to their source values, are often
found in software like parsers and parser generators, language interpreters,
and compilers; they can be used whenever a given algorithm compares its inputs
by identity only.

## Examples

As we would expect, interning works well with string types.

```rust
// Import Symtern's traits, which allow us to use each interner the same way
// regardless of the underlying implementation.
use symtern::prelude::*;

// Create a new pool that accepts `&str` arguments to `intern`, and uses
// `u8` as the backing representation for its symbol type.
let mut pool = symtern::Pool::<str,u8>::new();
if let (Ok(hello), Ok(world)) = (pool.intern("Hello"), pool.intern("World")) {
    assert!(hello != world);

    assert_eq!(hello, hello);
    assert_eq!(Ok(hello), pool.intern("Hello"));
    assert_eq!(Ok("Hello"), pool.resolve(hello));

    assert_eq!(world, world);
    assert_eq!(Ok(world), pool.intern("World"));
    assert_eq!(Ok("World"), pool.resolve(world));
}
```

### Error handling

The type used to identify a given interner's symbols can represent only
a finite range of values; because of this you must allow for the possibility
that any attempt to intern a value may fail.

```rust
use symtern::prelude::*;
use symtern::Pool;
use symtern::ErrorKind;

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
```

It's also possible for a symbol to fail to resolve; in this case the error's
`.kind()` method will return `ErrorKind::NoSuchSymbol`.

## API Stability

This library's API has not yet been stabilized; specifically, there are
compromises to be considered with respect to safety and complexity.  Some of
these issues are discussed in [src/traits.rs](src/traits.rs).

## Contributing

Find a bug?  File an issue!  Have an idea for a feature or improvement?
We'd love to hear about it; fork the repository and make your changes, then
submit a pull request for review.

## License

Symtern is dual-licensed under either the
[Apache License version 2.0](LICENSE-Apache) or the [MIT License](LICENSE-MIT).
