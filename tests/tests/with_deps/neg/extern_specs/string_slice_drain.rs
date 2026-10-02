//@compile-flags: -Fstd-extern-specs=on -Fcheck-overflow=strict
use flux_rs::*;

// Numeric bounds alone do not establish a UTF-8 boundary.
#[spec(fn(&String[@s], end: usize) -> &str
    requires end <= flux_core::str::byte_len(s))]
fn numeric_bound(text: &String, end: usize) -> &str {
    &text[..end] //~ ERROR refinement type
}

fn unchecked_drain(text: &mut String, end: usize) {
    text.drain(..end); //~ ERROR refinement type
                      //~| ERROR refinement type
}

#[spec(fn(text: &mut String[@s], end: usize)
    requires flux_core::str::boundary(s, end)
    ensures text: String{out:
        flux_core::str::byte_len(out) == flux_core::str::byte_len(s) - end})]
fn cannot_assume_removal(text: &mut String, end: usize) { //~ ERROR refinement type
    std::mem::forget(text.drain(..end));
}

// Custom RangeBounds may run user code; the prefix capability is closed.
struct Custom;
#[assoc(fn prefix_end(range: Self) -> int { 0 })]
impl std::ops::RangeBounds<usize> for Custom {
    fn start_bound(&self) -> std::ops::Bound<&usize> { std::ops::Bound::Unbounded }
    fn end_bound(&self) -> std::ops::Bound<&usize> { std::ops::Bound::Unbounded }
}
fn custom_range(text: &mut String) {
    text.drain(Custom); //~ ERROR refinement type
}
