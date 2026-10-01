use flux_attrs::*;

#[sig(fn<T>(value: T[@v]) -> T[v])]
fn identity<T>(value: T) -> T { value }

#[sig(fn(first: &i32[@a], second: &i32) -> i32[a])]
fn wrong_reference(first: &i32, second: &i32) -> i32 {
    *identity(second) //~ ERROR refinement type
}

#[sig(fn(first: &i32[@a], second: &i32[@b], choose: bool[@c]) -> i32[if c { b } else { a }])]
fn swapped_branch(first: &i32, second: &i32, choose: bool) -> i32 {
    let selected = if choose { first } else { second };
    *identity(selected) //~ ERROR refinement type
}

#[sig(fn(value: &mut i32[@v]) -> i32[v])]
fn mutated(value: &mut i32) -> i32 {
    *value = 7; //~ ERROR assignment might be unsafe
    *value
}
