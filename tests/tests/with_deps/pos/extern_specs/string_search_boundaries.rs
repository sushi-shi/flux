//@compile-flags: -Fstd-extern-specs=on -Fcheck-overflow=strict -Fno-panic=on
use flux_rs::*;

#[spec(fn(&str[@text], &str) -> Option<usize{end: flux_core::str::boundary(text, end)}>)]
fn match_end(text: &str, needle: &str) -> Option<usize> {
    text.find(needle).map(|start| start + needle.len())
}

#[spec(fn(&str[@text], &str) -> Option<usize{start: flux_core::str::boundary(text, start)}>)]
fn suffix_start(text: &str, needle: &str) -> Option<usize> {
    if text.ends_with(needle) {
        Some(text.len() - needle.len())
    } else {
        None
    }
}

// The generic pattern argument must retain which reference was passed.
#[spec(fn(&str[@text], &str, &str) -> Option<usize{end: flux_core::str::boundary(text, end)}>)]
fn choose_needle(text: &str, first: &str, second: &str) -> Option<usize> {
    let needle = if first.is_empty() { second } else { first };
    text.find(needle).map(|start| start + needle.len())
}
