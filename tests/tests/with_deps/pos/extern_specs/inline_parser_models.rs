//@compile-flags: -Fstd-extern-specs=on -Fcheck-overflow=strict -Fno-panic=on
use flux_core::{ensures, may_panic, requires};

#[requires(lo <= hi)]
#[ensures(lo <= result && result <= hi)]
fn reverse_bounds(lo: usize, hi: usize) -> usize {
    (lo..=hi).rev().next().unwrap_or(lo)
}

fn alternating_ends(lo: i32, hi: i32) {
    let mut range = lo..=hi;
    if let Some(value) = range.next_back() { assert!(lo <= value && value <= hi); }
    if let Some(value) = range.next() { assert!(lo <= value && value <= hi); }
    for value in range.rev() { assert!(lo <= value && value <= hi); }
}

#[ensures(needle.is_char_boundary(result))]
#[ensures(result <= text.len() && (needle.is_empty() || result < needle.len()))]
fn suffix(text: &str, needle: &str) -> usize {
    let max = text.len().min(needle.len().saturating_sub(1));
    for k in (1..=max).rev() {
        if needle.is_char_boundary(k) && text.ends_with(&needle[..k]) { return k; }
    }
    0
}

#[may_panic]
#[ensures(result == text)]
fn copy(text: &str) -> String {
    let mut out = String::new();
    out.push_str(text);
    out
}

#[flux::sig(fn(text: &strg String) ensures text: String[""])]
fn clear(text: &mut String) { text.clear(); }
