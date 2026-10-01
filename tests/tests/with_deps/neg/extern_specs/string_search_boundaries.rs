//@compile-flags: -Fstd-extern-specs=on -Fcheck-overflow=strict
use flux_rs::*;

#[spec(fn(&str[@text], &str, &str) -> Option<usize{end: flux_core::str::boundary(text, end)}>)]
fn wrong_needle(text: &str, needle: &str, other: &str) -> Option<usize> {
    text.find(needle).map(|start| start + other.len()) //~ ERROR refinement type
                                                     //~| ERROR refinement type
}

#[spec(fn(&str[@text], &str) -> Option<usize{start: flux_core::str::boundary(text, start)}>)]
fn no_suffix_check(text: &str, needle: &str) -> Option<usize> {
    Some(text.len().saturating_sub(needle.len())) //~ ERROR refinement type
}
