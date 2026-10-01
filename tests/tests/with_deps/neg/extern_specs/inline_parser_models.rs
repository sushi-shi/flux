//@compile-flags: -Fstd-extern-specs=on -Fcheck-overflow=strict -Fno-panic=on
use flux_core::{ensures, may_panic};

fn inclusive_is_not_exclusive() {
    for value in (3..=3usize).rev() { assert!(value < 3); } //~ ERROR may panic
}

fn unsupported_back<I: DoubleEndedIterator>(iter: I) {
    let _ = iter.rev().next(); //~ ERROR refinement type
    //~^ ERROR may panic
}

fn no_false_size_capability() {
    let _: Vec<_> = (1..=3usize).rev().collect(); //~ ERROR refinement type
    //~^ ERROR may panic
}

fn arbitrary_pattern_may_panic(text: &str) {
    let _ = text.ends_with(|_: char| panic!("predicate")); //~ ERROR refinement type
    //~^ ERROR may panic
}

#[may_panic]
#[ensures(result == "")]
fn appending_is_not_clearing(text: &str) -> String {
    let mut out = String::new();
    out.push_str(text);
    out //~ ERROR refinement type
}
