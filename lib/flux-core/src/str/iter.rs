//! Iterators over `str`.
//!
//! The `Iterator` contract in `iter::traits::iterator` says every `Some` shrinks `size` by one.
//! An iterator type without a refinement cannot honor that: all of its values share the same
//! (unit) index, so `size(curr) == size(next)` and the contract collapses to
//! `size(x) == size(x) - 1`. Anything that relies on it -- generic code over `I: Iterator`, or
//! adapters such as `Enumerate` -- is then vacuously verified. The iterators below are refined by
//! the number of items they have left: unknown and non-negative, but fixed when the iterator is
//! created, which is all the generic counting contract needs. `CharIndices` also
//! carries its original text and front byte offset to justify UTF-8 slicing.

use flux_attrs::*;

#[extern_spec(core::str)]
#[refined_by(size: int)]
#[invariant(size >= 0)]
struct Chars<'a>;

#[extern_spec(core::str)]
#[assoc(
    fn has_size_model() -> bool { true }
    fn size(x: Chars) -> int { x.size }
    fn done(x: Chars) -> bool { x.size <= 0 }
    fn step(x: Chars, y: Chars) -> bool {
        y.size == if x.size > 0 { x.size - 1 } else { x.size }
    }
)]
impl<'a> Iterator for Chars<'a> {
    #[spec(
        fn(self: &mut Self[@curr_s]) -> Option<_>[!<Self as Iterator>::done(curr_s)]
        ensures self: Self{next_s: <Self as Iterator>::step(curr_s, next_s)}
    )]
    fn next(&mut self) -> Option<char>;
}

#[extern_spec(core::str)]
#[refined_by(size: int, text: str, offset: int)]
#[invariant(size >= 0)]
#[invariant(super::boundary(text, offset))]
#[invariant(super::byte_len(text) <= isize::MAX)]
struct CharIndices<'a>;

#[extern_spec(core::str)]
#[assoc(
    fn has_size_model() -> bool { true }
    fn size(x: CharIndices) -> int { x.size }
    fn done(x: CharIndices) -> bool { x.size <= 0 }
    fn step(x: CharIndices, y: CharIndices) -> bool {
        y.text == x.text
        && y.size == if x.size > 0 { x.size - 1 } else { x.size }
        && (if x.size > 0 { y.offset > x.offset } else { y.offset == x.offset })
    }
)]
impl<'a> Iterator for CharIndices<'a> {
    // core/src/str/iter.rs: next returns front_offset, then advances it by the
    // consumed character's byte width. Both offsets are UTF-8 boundaries.
    // This abstraction preserves progress but not the exact next-offset/width
    // relation, so it does not yet prove complete character accounting.
    #[no_panic]
    #[spec(
        fn(self: &mut Self[@curr_s]) -> Option<
            (usize[curr_s.offset], char{c:
                super::boundary(curr_s.text, curr_s.offset)
                && super::boundary(curr_s.text, curr_s.offset + super::utf8_width(c))
            })>[!<Self as Iterator>::done(curr_s)]
        ensures self: Self{next_s: <Self as Iterator>::step(curr_s, next_s)}
    )]
    fn next(&mut self) -> Option<(usize, char)>;
}

#[extern_spec(core::str)]
#[refined_by(size: int)]
#[invariant(size >= 0)]
struct Lines<'a>;

#[extern_spec(core::str)]
#[assoc(
    fn has_size_model() -> bool { true }
    fn size(x: Lines) -> int { x.size }
    fn done(x: Lines) -> bool { x.size <= 0 }
    fn step(x: Lines, y: Lines) -> bool {
        y.size == if x.size > 0 { x.size - 1 } else { x.size }
    }
)]
impl<'a> Iterator for Lines<'a> {
    #[spec(
        fn(self: &mut Self[@curr_s]) -> Option<_>[!<Self as Iterator>::done(curr_s)]
        ensures self: Self{next_s: <Self as Iterator>::step(curr_s, next_s)}
    )]
    fn next(&mut self) -> Option<&'a str>;
}
