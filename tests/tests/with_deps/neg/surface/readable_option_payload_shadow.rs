//@compile-flags: -Fstd-extern-specs=on
use flux_core::ensures;

#[flux::refined_by(is_some: bool)]
struct Option<T> {
    #[flux::field(bool[is_some])]
    present: bool,
    value: T,
}

// A user-defined type cannot gain the standard Option payload model by name.
#[ensures(result.is_none_or(|value| value > 0))] //~ ERROR incompatible refinement annotation
fn shadowed() -> Option<usize> { Option { present: true, value: 0 } }

#[flux::refined_by(len: int)]
struct Vec<T> {
    #[flux::field(usize[len])]
    count: usize,
    value: T,
}

impl<T> Vec<T> {
    fn len(&self) -> usize { 999 }
}

// Canonicalization applies to payload types as well as the outer Option.
#[ensures(result.is_none_or(|v| v.len() == 0))] //~ ERROR incompatible refinement annotation
fn shadowed_payload() -> std::option::Option<Vec<()>> {
    Some(Vec { count: 0, value: () })
}
