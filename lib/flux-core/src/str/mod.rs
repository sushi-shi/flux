//! Extern specs for [`core::str`].

// Rust offsets count UTF-8 bytes; SMT's str_len counts characters. Keep byte
// length separate instead of identifying these two different quantities.
#![flux::defs {
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
}]

use flux_attrs::*;

#[extern_spec]
impl str {
    #[no_panic]
    #[spec(fn(&str[@s]) -> usize{n: n == byte_len(s) && n <= isize::MAX})]
    fn len(&self) -> usize;

    #[no_panic]
    #[spec(fn(&str[@s]) -> bool[byte_len(s) == 0])]
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
