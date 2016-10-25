// Copyright (C) 2016 Symtern Project Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-Apache
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. This file may not be copied, modified, or
// distributed except according to those terms.
struct MyInterner {
    // ...
    map: ::std::collections::HashMap<String,usize>,
    id_to_string: Vec<String>,
}
struct Sym<'a> {
    marker: ::std::marker::PhantomData<&'a ()>,
    // ...
    id: usize
}
impl MyInterner {
    fn new() -> Self {
        // ...
        MyInterner{map: ::std::collections::HashMap::new(), id_to_string: Vec::new()}
    }
    fn intern<'a>(&'a mut self, s: &str) -> Sym<'a> {
        // ...
        use ::std::marker::PhantomData;
        if self.map.contains_key(s) { Sym{marker: PhantomData, id: *self.map.get(s).unwrap()} }
        else {
            self.id_to_string.push(s.to_owned());
            let id = self.id_to_string.len() - 1;
            self.map.insert(self.id_to_string[id].clone(), id);
            Sym{marker: PhantomData, id: id}
        }
    }
}

fn main() {
    let mut interner = MyInterner::new();
    let x = interner.intern("x");
    let y = interner.intern("y");        //~ ERROR cannot borrow `interner` as mutable more than once at a time
}
