//@compile-flags: -Fstd-extern-specs=on -Fcheck-overflow=strict
use flux_core::{ensures, may_panic, requires};

#[requires(condition)]
fn check(condition: bool) {}

#[may_panic]
#[ensures(result.is_none_or(|index| index < limit))]
fn first_mapped_index(limit: usize) -> Option<usize> {
    (0..limit).map(|index| index).next()
}

#[may_panic]
fn callback_uses_range(limit: usize) {
    (0..limit).map(|index| check(index < limit)).next();
}

#[may_panic]
fn callback_in_loop(limit: usize) {
    for index in (0..limit).map(|index| index) {
        check(index < limit);
    }
}

#[may_panic]
#[ensures(result.is_none_or(|value| value == 5))]
fn constant_result(limit: usize) -> Option<usize> {
    (0..limit).map(|_| 5).next()
}

#[may_panic]
#[ensures(result.is_none())]
fn empty_range() -> Option<usize> {
    (0..0).map(|index| index).next()
}
