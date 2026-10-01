use flux_attrs::*;

#[sig(fn<T>(value: T[@v]) -> T[v])]
fn identity<T>(value: T) -> T { value }

#[sig(fn(value: &i32[@v]) -> i32[v])]
fn generic_reference(value: &i32) -> i32 { *identity(value) }

#[sig(fn(first: &i32[@a], second: &i32[@b], choose: bool[@c]) -> i32[if c { a } else { b }])]
fn branch(first: &i32, second: &i32, choose: bool) -> i32 {
    let selected = if choose { first } else { second };
    *identity(selected)
}

#[sig(fn(value: & &i32[@v]) -> i32[v])]
fn nested(value: &&i32) -> i32 { **identity(value) }

// Filling this return hole must remap outer lifetimes beneath newly indexed
// reference binders without counting the hole's own binders twice.
#[sig(fn(value: &i32) -> _)]
fn inferred_return(value: &i32) -> &i32 { value }

#[sig(fn(x: &mut i32) ensures x: i32[2])]
fn mutable_reference_still_changes(x: &mut i32) {
    *x = 1;
    *x = 2;
}
