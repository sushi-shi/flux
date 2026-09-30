use flux_rs::attrs::*;

#[sig(fn(x: i32) -> i32[x])]
pub fn good(x: i32) -> i32 { x }

#[sig(fn() -> i32[1])]
pub fn bad() -> i32 { 0 }

#[trusted]
pub fn trusted() -> i32 { 42 }

#[flux_rs::ignore]
pub fn ignored() -> i32 { 42 }

pub trait Declaration { fn no_body(&self); }

pub fn enclosing() -> i32 { let f = || 1; f() }
