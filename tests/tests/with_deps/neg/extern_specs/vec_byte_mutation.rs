//@compile-flags: -Fstd-extern-specs=on -Fcheck-overflow=strict -Fno-panic=on
#![feature(drain_keep_rest)]

#[flux::sig(fn(v: &strg Vec<u8>[4]) ensures v: Vec<u8>{n: n <= 2})]
fn leak_is_not_exact_removal(v: &mut Vec<u8>) {
    std::mem::forget(v.drain(..2));
    assert!(v.len() == 2); //~ ERROR may panic
}

fn cannot_restore_drained_bytes(v: &mut Vec<u8>) {
    v.drain(..0).keep_rest(); //~ ERROR refinement type
    //~^ ERROR may panic
}

fn unknown_destructor<T>(v: &mut Vec<T>) { v.clear(); } //~ ERROR may panic

fn unknown_clone<T: Clone>(v: &mut Vec<T>, bytes: &[T]) {
    v.extend_from_slice(bytes); //~ ERROR refinement type
    //~^ ERROR may panic
}

#[flux::sig(fn(v: &mut Vec<u8>[4]))]
fn past_end(v: &mut Vec<u8>) { v.drain(..5); } //~ ERROR refinement type
