extern crate flux_alloc;
extern crate flux_core;
use std::collections::BTreeMap;

#[flux_core::refined]
#[flux::invariant(bytes == pending.total)]
struct Buffer { pending: BTreeMap<u32, Vec<u8>>, bytes: usize }

// Unmodeled mutable access must forget the old map facts. These checks do not
// enable no-panic: rejection must come from accounting, not a missing effect model.
fn mutate_value(buffer: &mut Buffer) {
    if let Some(value) = buffer.pending.get_mut(&0) { //~ ERROR type invariant
        value.push(0);
    } //~ ERROR type invariant
}

fn clear_without_charging(buffer: &mut Buffer) {
    buffer.pending.clear();
} //~ ERROR type invariant
