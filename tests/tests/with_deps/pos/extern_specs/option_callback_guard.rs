//@compile-flags: -Fstd-extern-specs=on -Fcheck-overflow=strict -Fno-panic=on
use flux_core::{ensures, may_panic};

// The callback runs only for Some, which establishes the captured slice bound.
// There are no handwritten preconditions or callback contracts.
#[ensures(result.is_some() == !xs.is_empty())]
fn optional_head(xs: &[u8]) -> Option<u8> {
    let requested = if xs.is_empty() { None } else { Some(()) };
    requested.map(|_| xs[0])
}

fn head_or_default(xs: &[u8]) -> u8 {
    let requested = if xs.is_empty() { None } else { Some(()) };
    requested.map_or(0, |_| xs[0])
}

fn optional_nonzero_head(xs: &[u8]) -> Option<u8> {
    let requested = if xs.is_empty() { None } else { Some(()) };
    requested.and_then(|_| {
        let head = xs[0];
        if head == 0 { None } else { Some(head) }
    })
}

// Reduction of the Codex parser's index-selection proof: an empty slice
// produces None, so its callback has no index-bound obligation.
#[may_panic]
#[ensures(result.is_none_or(|index| index < xs.len()))]
fn first_index(xs: &[u8]) -> Option<usize> {
    xs.iter().enumerate().next().map(|(index, _)| index)
}

#[ensures(result.is_none())]
fn absent_head() -> Option<u8> {
    let absent: Option<()> = None;
    let empty: &[u8] = &[];
    absent.map(|_| empty[0])
}
