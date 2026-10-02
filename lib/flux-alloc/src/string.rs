use std::fmt::Display;

extern crate alloc;

use flux_attrs::*;

/// A closed-by-default capability for the standard ToString blanket impl.
/// Unknown Display implementations may panic or return fmt::Error.
/// Opting in is an external model obligation, not a proof about arbitrary fmt.
#[extern_spec(core::fmt)]
#[assoc(fn string_conversion_no_panic() -> bool { false })]
trait Display {}

/// Audited against pinned Rust 8925ea358a0f (alloc/src/string.rs): the
/// `to_string_str!` specialization for str calls String::from(&str), copying an
/// existing, valid allocation-sized slice. It invokes no user Display code.
/// Allocation failure can abort; this model only rules out Rust panics.
#[extern_spec(core::fmt)]
#[assoc(fn string_conversion_no_panic() -> bool { true })]
impl Display for str {}

#[extern_spec(alloc::string)]
impl<T: Display + ?Sized> ToString for T {
    #[no_panic_if(<T as Display>::string_conversion_no_panic())]
    #[spec(fn(&T) -> String)]
    fn to_string(&self) -> String;
}

#[extern_spec]
#[flux::refined_by(val: str)]
#[invariant(0 <= flux_core::str::byte_len(val) && flux_core::str::byte_len(val) <= isize::MAX)]
struct String;

#[extern_spec]
impl String {
    #[no_panic]
    #[spec(fn() -> String[""])]
    const fn new() -> String;

    #[no_panic]
    #[spec(fn(&String[@s]) -> usize[flux_core::str::byte_len(s)])]
    const fn len(&self) -> usize;

    #[no_panic]
    #[spec(fn(&String[@s]) -> bool[s == ""])]
    const fn is_empty(&self) -> bool;

    #[no_panic]
    #[spec(fn(self: &mut String) ensures self: String[""])]
    fn clear(&mut self);

    // Normal-return content/length facts; capacity growth can panic.
    #[spec(fn(self: &mut String[@before], &str[@text])
        ensures self: String{after:
            after == str_concat(before, text)
            && flux_core::str::byte_len(after) == flux_core::str::byte_len(before)
                + flux_core::str::byte_len(text)})]
    fn push_str(&mut self, string: &str);

    // Pinned alloc/src/string.rs removes the range in Drain::drop. Leaking
    // the iterator leaves the original string; consuming it, even partly,
    // then dropping it removes the whole prefix. Both outcomes must be
    // admitted until the checker tracks destructor effects on the owner.
    #[no_panic_if(<R as core::ops::RangeBounds<usize>>::modeled_prefix())]
    #[spec(fn(self: &mut String[@before], R[@range]) -> alloc::string::Drain
        requires <R as core::ops::RangeBounds<usize>>::modeled_prefix()
            && flux_core::str::boundary(before,
                <R as core::ops::RangeBounds<usize>>::prefix_end(range))
        ensures self: String{after: after == before || (
            str_suffix_of(after, before)
            && flux_core::str::byte_len(after) == flux_core::str::byte_len(before)
                - <R as core::ops::RangeBounds<usize>>::prefix_end(range))})]
    fn drain<R: core::ops::RangeBounds<usize>>(&mut self, range: R) -> alloc::string::Drain<'_>;
}

// String delegates indexing to the same sealed SliceIndex<str> operations
// as str. Keeping both the boundary requirement and output relation lets
// callers use the sliced text without a separate hand-written summary.
#[extern_spec]
impl<I: core::slice::SliceIndex<str>> core::ops::Index<I> for String {
    #[no_panic]
    #[spec(fn(&String[@text], {I[@idx] |
        <I as core::slice::SliceIndex<str>>::in_bounds(idx, text)})
        -> &I::Output{out: <I as core::slice::SliceIndex<str>>::output_pred(idx, text, out)})]
    fn index(&self, index: I) -> &I::Output;
}

#[extern_spec]
impl core::ops::Deref for String {
    #[no_panic]
    #[spec(fn(&String[@text]) -> &str[text])]
    fn deref(&self) -> &str;
}

#[extern_spec]
#[assoc(
    fn is_eq(x: String, y: String, res: bool) -> bool { res <=> (x.val == y.val) }
    fn is_ne(x: String, y: String, res: bool) -> bool { res <=> (x.val != y.val) }
)]
impl PartialEq for String {
    #[spec(fn(&String[@s], &String[@t]) -> bool[s == t])]
    fn eq(&self, other: &String) -> bool;
}

#[extern_spec]
#[assoc(fn cloned(old: Self, new: Self) -> bool { old == new } )]
impl Clone for String {
    #[spec(fn (&String[@s]) -> String[s])]
    fn clone(&self) -> Self;
}

#[trusted]
#[spec(fn(&str[@s]) -> String[s])]
pub fn mk_string(s: &str) -> String {
    s.to_string()
}
