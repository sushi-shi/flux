//@compile-flags: -Fcheck-overflow=strict -Fno-panic=on
extern crate flux_alloc;
extern crate flux_core;

#[flux::sig(fn(v: &mut Vec<()>[usize::MAX]))]
fn overfull_zst(v: &mut Vec<()>) { v.push(()); } //~ ERROR may panic

// One extra byte fits, but doubling a full allocation can still overflow.
#[flux::sig(fn(v: &strg Vec<u8>[isize::MAX / 2 + 1])
    ensures v: Vec<u8>[isize::MAX / 2 + 2])]
fn doubling_overflows(v: &mut Vec<u8>) { v.push(0); } //~ ERROR may panic

// A ZST vector can really have this length; its input must not be contradictory.
#[flux::sig(fn(v: &Vec<()>[usize::MAX]))]
fn zst_is_not_bounded_by_bytes(v: &Vec<()>) {
    assert!(v.len() <= isize::MAX as usize); //~ ERROR may panic
}
