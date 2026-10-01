//@compile-flags: -Fstd-extern-specs=on -Fcheck-overflow=strict -Fno-panic=on
use flux_rs::*;

#[spec(fn(&String[@s], end: usize) -> &str{part:
    flux_core::str::byte_len(part) == end && str_prefix_of(part, s)}
    requires flux_core::str::boundary(s, end))]
fn prefix(text: &String, end: usize) -> &str {
    &text[..end]
}

#[spec(fn(&String[@s], start: usize) -> &str{part:
    flux_core::str::byte_len(part) == flux_core::str::byte_len(s) - start
        && str_suffix_of(part, s)}
    requires flux_core::str::boundary(s, start))]
fn suffix(text: &String, start: usize) -> &str {
    &text[start..]
}

#[spec(fn(text: &mut String[@s], end: usize)
    requires flux_core::str::boundary(s, end)
    ensures text: String{out: out == s || (str_suffix_of(out, s)
        && flux_core::str::byte_len(out) == flux_core::str::byte_len(s) - end)})]
fn drain(text: &mut String, end: usize) {
    text.drain(..end);
}

// The same disjunction covers a forgotten iterator. Claiming unconditional
// removal here would be unsound because Drain applies its edit in Drop.
#[spec(fn(text: &mut String[@s], end: usize)
    requires flux_core::str::boundary(s, end)
    ensures text: String{out: out == s || (str_suffix_of(out, s)
        && flux_core::str::byte_len(out) == flux_core::str::byte_len(s) - end)})]
fn forget_drain(text: &mut String, end: usize) {
    std::mem::forget(text.drain(..end));
}
