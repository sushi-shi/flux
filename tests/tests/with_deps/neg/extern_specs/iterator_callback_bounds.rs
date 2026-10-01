//@compile-flags: -Fstd-extern-specs=on -Fcheck-overflow=strict
use flux_core::{ensures, may_panic, requires};

#[requires(condition)]
fn check(condition: bool) {}

#[may_panic]
#[ensures(result.is_none_or(|index| index < limit))]
fn wrong_output(limit: usize) -> Option<usize> {
    (0..limit).map(|_| limit).next() //~ ERROR refinement type error
}

#[may_panic]
fn wrong_callback(limit: usize) {
    (0..limit).map(|index| check(index >= limit)).next(); //~ ERROR refinement type error
}

#[may_panic]
fn inhabited_loop(limit: usize) {
    for _ in (0..limit).map(|index| index) {
        check(false); //~ ERROR refinement type error
    }
}

#[may_panic]
#[ensures(result.is_none_or(|value| value == 6))]
fn wrong_constant(limit: usize) -> Option<usize> {
    (0..limit).map(|_| 5).next() //~ ERROR refinement type error
}

#[may_panic]
#[ensures(result.is_some())]
fn empty_is_not_present() -> Option<usize> {
    (0..0).map(|index| index).next() //~ ERROR refinement type error
}
