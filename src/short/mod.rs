//! String interner optimized for short strings.
//!
//! This module, initially based on [Servo's string-cache
//! library](https://github.com/servo/string-cache), provides a hash-based
//! string-interning implementation that utilizes the small-string optimization
//! when possible.
//!
//! Unline Servo's string-cache library, this module leverages Rust's
//! lifetime-safety guarantees &mdash; instead of mutex-protected static
//! globals &mdash; to safely tie symbols to the lifetimes of their parent
//! symbol pools, which allows creation of multiple independent symbol pools.
use std::fmt;
use std::convert::AsRef;
use std::marker::PhantomData;
use std::hash::{Hash, Hasher};
use std::collections::BTreeMap;
use std::default::Default;
use std::sync::Mutex;

use traits::{Interner, SelfResolver};
use Result;

use self::private::PackFormat;

mod private;                    // implementation details

#[cfg(test)] mod tests;


#[cfg(feature = "fnv")]
type HashAlgo = ::fnv::FnvHasher;

#[cfg(not(feature = "fnv"))]
type HashAlgo = ::std::collections::hash_map::DefaultHasher;

impl<'a, T: ?Sized> Interner<'a, T> for Pool
    where T: AsRef<str>
{
    type Symbol = Sym<'a>;
    fn intern(&'a self, value: &T) -> Result<Self::Symbol> {
        Ok(self.sym(value))
    }
}

impl<'a> SelfResolver for Sym<'a> {
    type Stored = str;
    fn resolve(&self) -> &str {
        self.as_ref()
    }
}


/* ****************************************************************
 * Pool
 */
/// A collection of symbols.
///
/// See [the module-level documentation](index.html) for more information.
#[derive(Debug)]
pub struct Pool {
    map:  Mutex<BTreeMap<u64,String>>
}

impl Clone for Pool {
    fn clone(&self) -> Self {
        let m = self.map.lock().expect("Failed to lock symbol pool mutex for clone");
        Pool{map: Mutex::new(m.clone())}
    }
}

impl Pool {
    /// Create a new symbol pool.
    #[inline]
    pub fn new() -> Pool {
        Pool{map: Mutex::new(BTreeMap::new())}
    }

    /// Fetch a symbol corresponding to the given string.
    #[inline(always)]
    pub fn sym<'a, S: AsRef<str>>(&'a self, name: S) -> Sym<'a> {
        Sym(unsafe { self.symbol(name) }, PhantomData)
    }

    /// Fetch a lifetime-unsafe symbol corresponding to the given string.
    ///
    /// Care must be taken to ensure that the returned symbol is not used after
    /// the pool goes out of scope.
    ///
    /// This method should remain private, and shouldn't be used unless you're
    /// absolutely certain it's what you want.
    unsafe fn symbol<S: AsRef<str>>(&self, name: S) -> private::Symbol {
        let name = name.as_ref();
        if name.len() <= private::INLINE_SYMBOL_MAX_LEN {
            private::Inline::new(name).pack()
        } else {
            let hsh = { let mut h = HashAlgo::default();
                        name.hash(&mut h);
                        h.finish() } << 1;

            let mut map = self.map.lock().expect("Failed to lock symbol pool mutex for insertion");
            if ! map.contains_key(&hsh) {
                map.insert(hsh, name.to_owned());
            }
            private::Pooled::new(hsh, self).pack()
        }
    }
}

impl Default for Pool {
    #[inline]
    fn default() -> Self {
        Pool{map: Default::default()}
    }
}

/* ****************************************************************
 * Symbol
 */
/// Symbol obtained from a symbol pool.
///
/// Instances of `Sym` are created using the `sym` method on
/// [`Pool`](struct.Pool.html).  See [the module-level
/// documentation](index.html) for more information.

// `Sym` is a safe wrapper around the internal `private::Symbol`, which does not carry
// a lifetime parameter and therefore cannot ensure that any pool reference
// carried by an instance is still valid.  By adding a lifetime parameter and
// appropriate `PhantomData` data member, `Sym` prevents this case.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct Sym<'a>(private::Symbol, PhantomData<&'a Pool>);

impl<'a> fmt::Display for Sym<'a> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <private::Symbol as fmt::Display>::fmt(&self.0, f)
    }
}

impl<'a> AsRef<str> for Sym<'a> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl<'a> Hash for Sym<'a>  {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
