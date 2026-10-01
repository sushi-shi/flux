//! Bounds-only inclusive ranges. The indices are an immutable envelope, not
//! the current endpoints. No exact size or exhaustion capability is granted.

use core::iter::Step;

use flux_attrs::*;

#[extern_spec(core::ops)]
#[refined_by(lower: Idx, upper: Idx)]
struct RangeInclusive<Idx>;

#[extern_spec(core::ops)]
impl<Idx> RangeInclusive<Idx> {
    #[no_panic]
    #[spec(fn(start: Idx, end: Idx) -> RangeInclusive<Idx>[start, end])]
    const fn new(start: Idx, end: Idx) -> Self;
}

#[extern_spec(core::ops)]
#[assoc(fn valid_item(range: Self, item: A) -> bool {
    range.lower <= item && item <= range.upper
})]
impl<A: Step> Iterator for RangeInclusive<A> {
    #[no_panic_if(<A as Step>::ordered_no_panic())]
    #[spec(fn(self: &mut Self[@range]) -> Option<A{v: range.lower <= v && v <= range.upper}>
        requires <A as Step>::ordered_no_panic()
        ensures self: Self[range])]
    fn next(&mut self) -> Option<A>;
}

#[extern_spec(core::ops)]
#[assoc(
    fn modeled_back() -> bool { <A as Step>::ordered_no_panic() }
    fn back_no_panic() -> bool { <A as Step>::ordered_no_panic() }
    fn back_step(before: Self, after: Self) -> bool { before == after }
)]
impl<A: Step> DoubleEndedIterator for RangeInclusive<A> {
    #[no_panic_if(<A as Step>::ordered_no_panic())]
    #[spec(fn(self: &mut Self[@range]) -> Option<A{v: range.lower <= v && v <= range.upper}>
        requires <A as Step>::ordered_no_panic()
        ensures self: Self[range])]
    fn next_back(&mut self) -> Option<A>;
}
