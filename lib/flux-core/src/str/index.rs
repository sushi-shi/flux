use core::ops;

use flux_attrs::*;

// core/src/str/traits.rs delegates Index and IndexMut to sealed SliceIndex.
// The range implementations check UTF-8 boundaries as well as numeric bounds.
// Use `str` explicitly below: Self in a generic primitive extern impl currently
// lowers to a free alias without the impl arguments and crashes normalization.

#[extern_spec(core::str)]
impl<I: core::slice::SliceIndex<str>> ops::Index<I> for str {
    #![assoc(
        fn in_bounds(text: str, idx: I) -> bool {
            <I as core::slice::SliceIndex<str>>::in_bounds(idx, text)
        }
        fn output_pred(text: str, idx: I, out: <I as core::slice::SliceIndex<str>>::Output) -> bool {
            <I as core::slice::SliceIndex<str>>::output_pred(idx, text, out)
        }
    )]
    #[no_panic]
    #[spec(fn(&str[@s], {I[@idx] | <str as ops::Index<I>>::in_bounds(s, idx)})
        -> &I::Output{out: <str as ops::Index<I>>::output_pred(s, idx, out)})]
    fn index(&self, index: I) -> &I::Output;
}

#[extern_spec(core::str)]
impl<I: core::slice::SliceIndex<str>> ops::IndexMut<I> for str {
    #[no_panic]
    #[spec(fn(&mut str[@s], {I[@idx] | <str as ops::Index<I>>::in_bounds(s, idx)})
        -> &mut I::Output{out: <str as ops::Index<I>>::output_pred(s, idx, out)})]
    fn index_mut(&mut self, index: I) -> &mut I::Output;
}

#[extern_spec(core::str)]
#[assoc(fn in_bounds(r: Self, s: str) -> bool { super::boundary(s, r.end) })]
#[assoc(fn output_pred(r: Self, s: str, out: str) -> bool {
    super::byte_len(out) == r.end && str_prefix_of(out, s)
})]
impl core::slice::SliceIndex<str> for ops::RangeTo<usize> {}

#[extern_spec(core::str)]
#[assoc(fn in_bounds(r: Self, s: str) -> bool { super::boundary(s, r.start) })]
#[assoc(fn output_pred(r: Self, s: str, out: str) -> bool {
    super::byte_len(out) == super::byte_len(s) - r.start && str_suffix_of(out, s)
})]
impl core::slice::SliceIndex<str> for ops::RangeFrom<usize> {}

#[extern_spec(core::str)]
#[assoc(fn in_bounds(r: Self, s: str) -> bool {
    r.start <= r.end && super::boundary(s, r.start) && super::boundary(s, r.end)
})]
#[assoc(fn output_pred(r: Self, s: str, out: str) -> bool {
    super::byte_len(out) == r.end - r.start && str_contains(s, out)
})]
impl core::slice::SliceIndex<str> for ops::Range<usize> {}

#[extern_spec(core::str)]
#[assoc(fn in_bounds(r: Self, s: str) -> bool { true })]
#[assoc(fn output_pred(r: Self, s: str, out: str) -> bool { out == s })]
impl core::slice::SliceIndex<str> for ops::RangeFull {}
