// Copyright (C) 2016 Symtern Project Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-Apache
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. This file may not be copied, modified, or
// distributed except according to those terms.
//! Items intended for glob-import.
//!
//! Symtern takes a conservative stance on what names should will be brought
//! into scope when you glob-import this module's contents: only traits
//! necessary for method resolution are included, and the names of all
//! re-exported traits are prefixed with `Symtern` to reduce the likelihood of
//! name collisions.
//!
//! This stance may change in future versions, but for now must write your own
//! `use` statements for any other Symtern types you wish to use.
// N.B. we're not using a brace-enclosed imports list here because it's harder
// to read when rendered by rustdoc.
pub use traits::Len as SymternLen;
pub use traits::Intern as SymternIntern;
pub use traits::Resolve as SymternResolve;
pub use traits::ResolveUnchecked as SymternResolveUnchecked;

