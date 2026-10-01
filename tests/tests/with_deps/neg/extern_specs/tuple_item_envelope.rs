//@compile-flags: -Fstd-extern-specs=on
use flux_rs::attrs::*;
use flux_rs::assert;
extern crate flux_core;

struct Wrong;

#[assoc(fn valid_item(self: Self, item: (int, bool)) -> bool {
    item.0 == 0 && item.1
})]
impl Iterator for Wrong {
    type Item = (usize, bool);

    #[spec(fn(self: &mut Self) -> Option<(usize, bool)> ensures self: Self)]
    fn next(&mut self) -> Option<(usize, bool)> { //~ ERROR refinement type
        //~| ERROR refinement type
        Some((1, false))
    }
}

fn wrong_callback() {
    // Even under this advertised item predicate, a different index is false.
    Wrong.for_each(|(index, _)| assert(index == 2)); //~ ERROR refinement type
}
