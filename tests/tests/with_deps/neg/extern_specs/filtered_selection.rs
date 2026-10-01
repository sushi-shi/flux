//@compile-flags: -Fstd-extern-specs=on -Fcheck-overflow=strict -Fno-panic=on
use flux_core::{ensures, may_panic};
#[may_panic]
#[ensures(result.is_none_or(|index| index < xs.len()))]
fn wrong_index(xs: &[u8]) -> Option<usize> {
    xs.iter().enumerate().filter_map(|_| Some(xs.len())).min_by(|a, b| a.cmp(b)) //~ ERROR refinement type
}

#[may_panic]
#[ensures(result.is_none_or(|flag| flag))]
fn wrong_boolean(n: usize) -> Option<bool> {
    (0..n).filter_map(|i| Some(i >= n)).min_by(|_, _| core::cmp::Ordering::Equal) //~ ERROR refinement type
}

#[may_panic]
#[ensures(result.is_none_or(|(_, index)| index < xs.len()))]
fn wrong_tuple(xs: &[u8]) -> Option<(usize, usize)> {
    xs.iter().enumerate().filter_map(|(i, _)| Some((i, xs.len()))).min_by(|a, b| a.0.cmp(&b.0)) //~ ERROR refinement type
}

fn unchecked_comparator(xs: &[u8]) -> core::cmp::Ordering {
    core::cmp::Ordering::Equal.then_with(|| xs[0].cmp(&0)) //~ ERROR possible out-of-bounds access
}

#[may_panic]
#[ensures(result.is_none_or(|index| index > text.len()))]
fn wrong_search_offset(text: &str, needle: &str) -> Option<usize> {
    text.find(needle) //~ ERROR refinement type
}
