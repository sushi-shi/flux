//@compile-flags: -Fstd-extern-specs=on -Fcheck-overflow=strict -Fno-panic=on
// No extern crate flux_alloc: typed field projections must find forced model crates.
use flux_core::{ensures, may_panic, refined};

#[refined(buffer)]
struct Parser<T> { inner: T, buffer: Vec<u8> }

impl<T> Parser<T> {
    #[ensures(result.buffer.is_empty())]
    fn new(inner: T) -> Self { Self { inner, buffer: Vec::new() } }

    #[ensures(result == self.buffer.len())]
    fn buffered(&self) -> usize { self.buffer.len() }

    #[ensures(!old(self.buffer.is_empty()) || result.is_ok())]
    fn into_inner(self) -> Result<T, ()> {
        if self.buffer.is_empty() { Ok(self.inner) } else { Err(()) }
    }
}

#[ensures(result.buffer.is_empty())]
fn construct<T>(inner: T) -> Parser<T> { Parser::new(inner) }

#[may_panic]
#[ensures(result == 0)]
fn callback<F: FnOnce()>(f: F) -> u32 { f(); 0 }
