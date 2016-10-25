//! Example for lib.rs to demonstrate that symbol types implement Copy.
extern crate symtern;
use symtern::basic::Pool;
use symtern::traits::*;

/// Take ownership of a value, consuming it.
fn consume<T>(_: T) {}

fn main() {
    let mut pool = Pool::<str, u32>::new();
    let sym = pool.intern("xyz").unwrap();
    consume(sym);
    println!("The symbol is still valid: {:?}", pool.resolve(sym));
}
