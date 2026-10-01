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

struct Active { close: &'static str }

// Reduced from Codex's streaming close-tag path: preserve the borrowed
// delimiter value through an inferred callback and an Option state change.
#[spec(fn(active: &mut Option<Active>, &str) -> bool[true])]
fn close_after_take(active: &mut Option<Active>, text: &str) -> bool {
    if let Some(close) = active.as_ref().map(|active| active.close) {
        if let Some(start) = text.find(close) {
            let Some(_completed) = active.take() else { return true; };
            return text.is_char_boundary(start + close.len());
        }
    }
    true
}
