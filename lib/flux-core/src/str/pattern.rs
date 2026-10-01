//! Literal string patterns only; arbitrary predicate patterns stay disabled.
use flux_attrs::*;

#[extern_spec(core::str::pattern)]
#[assoc(
    fn literal_string() -> bool { false }
    fn match_end(p: Self, text: str, offset: int) -> bool { true }
    fn suffix_result(p: Self, text: str, found: bool) -> bool { true }
)]
trait Pattern {}

#[extern_spec(core::str::pattern)]
#[assoc(
    fn literal_string() -> bool { true }
    // StrSearcher reports the complete literal's end at a UTF-8 boundary.
    fn match_end(p: Self, text: str, offset: int) -> bool {
        super::boundary(text, offset + super::byte_len(p))
    }
    // A matching valid UTF-8 suffix starts at a boundary in the haystack.
    fn suffix_result(p: Self, text: str, found: bool) -> bool {
        found == str_suffix_of(p, text)
            && (found => super::boundary(text, super::byte_len(text) - super::byte_len(p)))
    }
)]
impl<'b> Pattern for &'b str {}
