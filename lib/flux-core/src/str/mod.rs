//! Extern specs for [`core::str`].

// Rust offsets count UTF-8 bytes; SMT's str_len counts characters. Keep byte
// length separate instead of identifying these two different quantities.
#![cfg_attr(flux, flux::defs {
    fn nonempty_byte_len(s: str) -> int;
    fn byte_len(s: str) -> int { if s == "" { 0 } else { nonempty_byte_len(s) } }
    fn interior_boundary(s: str, offset: int) -> bool;
    fn boundary(s: str, offset: int) -> bool {
        0 <= offset && offset <= byte_len(s)
        && (offset == 0 || offset == byte_len(s) || interior_boundary(s, offset))
    }
    fn utf8_width(c: char) -> int {
        if c < '\u{80}' { 1 } else if c < '\u{800}' { 2 }
        else if c < '\u{10000}' { 3 } else { 4 }
    }
})]

use core::str::pattern::Pattern;

use flux_attrs::*;

#[extern_spec]
impl str {
    #[no_panic_if(<P as Pattern>::literal_string())]
    #[spec(fn(&str, P) -> bool
        requires <P as Pattern>::literal_string())]
    fn ends_with<P: Pattern>(&self, pat: P) -> bool
    where
        for<'a> P::Searcher<'a>: core::str::pattern::ReverseSearcher<'a>;

    #[no_panic]
    #[spec(fn(&str[@s]) -> usize{n: n == byte_len(s) && n <= isize::MAX})]
    fn len(&self) -> usize;

    #[no_panic]
    // Pinned core/src/str/mod.rs implements this as len() == 0. For valid
    // UTF-8, zero bytes also means the empty string, independently of the
    // byte/character length distinction for nonempty text.
    #[spec(fn(&str[@s]) -> bool{empty:
        empty == (byte_len(s) == 0) && empty == (s == "")})]
    fn is_empty(&self) -> bool;

    #[no_panic]
    #[spec(fn(&str[@s], offset: usize) -> bool[boundary(s, offset)])]
    fn is_char_boundary(&self, offset: usize) -> bool;

    #[no_panic]
    #[spec(fn(&str[@s]) -> core::str::CharIndices{it:
        it.text == s && it.offset == 0 && it.size <= byte_len(s)})]
    fn char_indices(&self) -> core::str::CharIndices<'_>;
}

#[extern_spec]
impl char {
    #[no_panic]
    #[spec(fn(c: char) -> usize[utf8_width(c)])]
    fn len_utf8(self) -> usize;
}

mod index;
mod iter;
mod pattern;
mod utf8;
