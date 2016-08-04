//! Tests for `symbol` utilities that require access to `symbol::private`

use super::Pool;
use super::private::{Type,Unpacked,Inline,PackFormat};

macro_rules! inline_test_strings {
    () => ((0u8..16u8).map(|i| (0u8..i).map(|j| (j  + 'a' as u8) as char).collect::<String>()));
}

#[test]
fn inline_as_slice() {
    for ref s in inline_test_strings!() {
        let sym = Inline::new(&s.as_ref());
        assert_eq!(sym.as_ref(), AsRef::<str>::as_ref(s));
    }
}

#[test]
fn inline_packed_as_slice() {
    for ref s in inline_test_strings!() {
        let inl = Inline::new(s.as_ref());
        assert_eq!(inl.as_ref(), AsRef::<str>::as_ref(s));
        assert_eq!(inl.pack().as_ref(), AsRef::<str>::as_ref(s));
    }
}


#[test]
fn inline_pack_unpack() {
    let foo = Inline::new("foo");
    assert_eq!(foo.as_ref(), "foo");

    let bar = foo.pack();

    println!("{:?}", foo);
    println!("{:?}", bar);


    panic_unless!(foo == foo, "each symbol::Inline instance should be equal to itself");
    panic_unless!(bar == bar, "each symbol::Symbol instance should be equal to itself");

    panic_unless!(Unpacked::Inline(foo) == bar.unpack(),
                  "pack-unpack cycle on a symbol::Inline should yield the same symbol");
}



#[test]
fn pooled_pack_unpack() {
    let pool = Pool::new();
    let a_str = "it was a very nice day";
    let b_str = "and everyone was happy";

    let a = pool.sym(a_str).0;
    let b = pool.sym(b_str).0;
    assert_eq!(a.type_of(), Type::POOLED);
    assert_eq!(b.type_of(), Type::POOLED);

    println!("{:?} => {:?}", a, a.as_ref());
    println!("{:?}", a.unpack());

    assert_eq!(a, a);
    panic_unless!(a != b && b != a, "`a` and `b` are distinct symbols and should not be equal");
    assert_eq!(a.as_ref(), a_str);
    assert_eq!(b.as_ref(), b_str);
}
