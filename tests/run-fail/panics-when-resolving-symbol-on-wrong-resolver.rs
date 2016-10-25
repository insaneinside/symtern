// error-pattern:Detected an invalid attempt to resolve a symbol
#![cfg(debug_assertions)]
extern crate symtern;
use symtern::traits::*;
use symtern::basic::Pool;

fn main() {
    let mut p1 = Pool::<str,u16>::new();
    let mut p2 = Pool::<str,u16>::new();

    let s1 = p1.intern("foo").unwrap();
    let s2 = p2.intern("bar").unwrap();

    p1.resolve(s2).unwrap();
}
