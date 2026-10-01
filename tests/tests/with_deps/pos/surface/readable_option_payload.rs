//@compile-flags: -Fstd-extern-specs=on -Fcheck-overflow=strict -Fno-panic=on
use flux_core::{ensures, refined};

#[ensures(result.is_none_or(|index| index < limit))]
fn last_index(limit: usize) -> Option<usize> {
    if limit == 0 { None } else { Some(limit - 1) }
}

#[ensures(result)]
fn caller(limit: usize) -> bool {
    match last_index(limit) {
        Some(index) => index < limit,
        None => true,
    }
}

#[ensures(result.is_none_or(|(position, index)| position <= limit && index < limit))]
#[ensures(result.is_some() == (limit > 0))]
fn pair(limit: usize) -> Option<(usize, usize)> {
    if limit == 0 { None } else { Some((limit, limit - 1)) }
}

#[ensures(result.is_none_or(|(_, index)| index < limit))]
#[ensures(result.is_none_or(|(position, _)| position <= limit))]
fn stacked(limit: usize) -> Option<(usize, usize)> { pair(limit) }

#[ensures(result.is_none_or(|(position, index)| { position <= limit && index < limit }))]
fn block_predicate(limit: usize) -> Option<(usize, usize)> { pair(limit) }

#[ensures(result.is_none_or(|_| false))]
#[ensures(result.is_none())]
fn absent<T>() -> Option<T> { None }

#[refined(items)]
struct Entries<T> { items: Vec<T> }

impl<T> Entries<T> {
    #[ensures(result.is_none_or(|(_, index)| index < self.items.len()))]
    fn find(&self) -> Option<(usize, usize)> {
        if self.items.is_empty() { None } else { Some((0, self.items.len() - 1)) }
    }
}

#[ensures(result.is_none_or(|limit| limit < 5))]
fn shadow(limit: usize) -> Option<usize> {
    if limit < 5 { Some(limit) } else { None }
}

#[refined]
struct Counter { value: usize }

#[ensures(counter.value == 0)]
#[ensures(result.is_none_or(|index| index < old(counter.value)))]
fn take_last(counter: &mut Counter) -> Option<usize> {
    let limit = counter.value;
    counter.value = 0;
    last_index(limit)
}

#[ensures(result.is_none_or(|items| items.len() == values.len()))]
fn optional_vector<T>(values: Vec<T>) -> Option<Vec<T>> { Some(values) }
