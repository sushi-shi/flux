//@compile-flags: -Fstd-extern-specs=on -Fcheck-overflow=strict -Fno-panic=on
use flux_core::{ensures, may_panic};

#[may_panic]
#[ensures(result == 1)]
fn false_postcondition() -> u32 { 0 } //~ ERROR refinement type

#[may_panic]
fn overflow_still_checked(x: u8) -> u8 { x + 1 } //~ ERROR arithmetic operation may overflow
