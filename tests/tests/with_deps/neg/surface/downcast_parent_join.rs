//@compile-flags: -Fstd-extern-specs=on -Fcheck-overflow=strict -Fno-panic=on
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
#[ensures(result.items.is_empty())]
fn false_empty<T>(add: bool, value: T) -> Chunk<Item<T>> {
    let mut out = Chunk::default();
    if add { out.items.push(Item { value }); }
    else { clear(&mut out); }
    out
} //~ ERROR refinement type

#[may_panic]
#[ensures(result.text == "wrong")]
fn false_text<T: Clone>(add: bool, mut remaining: usize, value: T) -> Chunk<Item<T>> {
    let mut out = Chunk::default();
    while remaining > 0 {
        if add { out.items.push(Item { value: value.clone() }); }
        else { clear(&mut out); }
        remaining -= 1;
    }
    out
} //~ ERROR refinement type

// Constructor templates must never become assumed payload predicates.
#[may_panic]
#[flux::sig(fn(bool, i32) -> Chunk<Item<i32{v: v > 0}>>)]
fn false_payload(add: bool, value: i32) -> Chunk<Item<i32>> {
    let mut out = Chunk::default();
    if add { out.items.push(Item { value }); }
    else { clear(&mut out); }
    out //~ ERROR refinement type
}
