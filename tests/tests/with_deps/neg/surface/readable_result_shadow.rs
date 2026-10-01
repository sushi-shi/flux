extern crate flux_core;
extern crate flux_alloc;
use flux_attrs::ensures;

#[flux::refined_by(is_ok: bool)]
struct Result<T, E> {
    #[flux::field(bool[is_ok])]
    flag: bool,
    value: T,
    error: E,
}

// A similarly named user type must not inherit the standard Result model.
#[ensures(result.is_ok())] //~ ERROR incompatible refinement annotation
fn shadowed() -> Result<(), ()> {
    Result { flag: true, value: (), error: () }
}

#[flux::refined_by(len: int)]
struct Vec<T> {
    #[flux::field(usize[len])]
    len: usize,
    value: T,
}

#[ensures(result.len() == 0)] //~ ERROR incompatible refinement annotation
fn shadowed_vector() -> Vec<()> { Vec { len: 0, value: () } }
