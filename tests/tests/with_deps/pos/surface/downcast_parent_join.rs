//@compile-flags: -Fstd-extern-specs=on -Fcheck-overflow=strict -Fno-panic=on
// Reduced from Codex's inline hidden-tag parser: one branch mutates a
// field, while another borrows the whole output and unpacks a fresh index.
use flux_core::{ensures, may_panic, refined};

#[refined(text, items)]
struct Chunk<T> { text: String, items: Vec<T> }
struct Item<T> { value: T }

impl<T> Default for Chunk<T> {
    #[ensures(result.text == "" && result.items.is_empty())]
    fn default() -> Self { Self { text: String::new(), items: Vec::new() } }
}

#[may_panic]
#[ensures(chunk.text == "")]
#[ensures(chunk.items == old(chunk.items))]
fn clear<T>(chunk: &mut Chunk<T>) { chunk.text.clear(); }

#[may_panic]
#[ensures(result.text == "")]
#[ensures(result.items.len() == if add { 1 } else { 0 })]
fn choose<T>(add: bool, value: T) -> Chunk<Item<T>> {
    let mut out = Chunk::default();
    if add { out.items.push(Item { value }); }
    else { clear(&mut out); }
    out
}

#[may_panic]
#[ensures(result.text == "")]
fn repeat<T: Clone>(add: bool, mut remaining: usize, value: T) -> Chunk<Item<T>> {
    let mut out = Chunk::default();
    while remaining > 0 {
        if add { out.items.push(Item { value: value.clone() }); }
        else { clear(&mut out); }
        remaining -= 1;
    }
    out
}

// The constructor's generic payload refinements must survive the same join.
#[may_panic]
#[flux::sig(fn(bool, i32{v: v > 0}) -> Chunk<Item<i32{v: v > 0}>>)]
fn positive_payload(add: bool, value: i32) -> Chunk<Item<i32>> {
    let mut out = Chunk::default();
    if add { out.items.push(Item { value }); }
    else { clear(&mut out); }
    out
}
