//@compile-flags: -Fcheck-overflow=strict -Fno-panic=on
extern crate flux_alloc;
extern crate flux_core;

use flux_core::{ensures, refined, requires};
use std::collections::BTreeMap;

#[derive(Default)]
#[refined]
#[flux::invariant(bytes == pending.total)]
struct Buffer {
    pending: BTreeMap<u32, Vec<u8>>,
    bytes: usize,
}

#[ensures(result.bytes == 0)]
fn empty() -> Buffer { Buffer { pending: BTreeMap::new(), bytes: 0 } }

#[requires(buffer.bytes <= 100)]
#[ensures(buffer.bytes <= old(buffer.bytes))]
fn remove(buffer: &mut Buffer, key: u32) {
    if let Some(payload) = buffer.pending.remove(&key) {
        buffer.bytes -= payload.len();
    }
}

#[refined]
struct Flags { count: u32, ready: bool }

#[ensures(flags.ready)]
#[ensures(flags.count == old(flags.count))]
fn ready(flags: &mut Flags) { flags.ready = true; }
