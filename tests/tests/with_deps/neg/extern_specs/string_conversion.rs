//@compile-flags: -Fno-panic=on
extern crate flux_alloc;
extern crate flux_core;
use std::fmt::{self, Display, Formatter};

struct Fails;
impl Display for Fails {
    fn fmt(&self, _: &mut Formatter<'_>) -> fmt::Result { Err(fmt::Error) }
}

fn failed_formatter() -> String { Fails.to_string() } //~ ERROR may panic
fn unknown_formatter<T: Display>(value: &T) -> String { value.to_string() } //~ ERROR may panic
