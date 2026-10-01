//@aux-build:inherited_callback_bounds_aux.rs
extern crate inherited_callback_bounds_aux;

use flux_attrs::*;
use inherited_callback_bounds_aux::Callbacks;

#[extern_spec]
impl<B, C, F: FnMut(usize) -> B, G: FnMut() -> C> Callbacks<F, G> {
    // The implementation invokes the first callback with zero. Its second
    // callback retains the inherited bound and must still constrain C.
    #[spec(fn(&mut Self) -> (B, C) where F: FnMut(usize[0]) -> B)]
    fn call_both(&mut self) -> (B, C);
}

#[spec(fn(bool[true]))]
fn check(condition: bool) {}

#[spec(fn() -> (usize, usize[7]))]
fn second_output_is_checked() -> (usize, usize) {
    Callbacks { first: |index| index, second: || 0 }.call_both() //~ ERROR refinement type error
}

fn first_input_is_checked() {
    Callbacks {
        first: |index| { check(index == 1); index }, //~ ERROR refinement type error
        second: || 7,
    }.call_both();
}
