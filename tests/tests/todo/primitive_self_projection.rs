// Blocker: Self inside a generic extern impl for a primitive is lowered as a
// free alias without the impl's generic arguments. Spell the receiver `str`
// explicitly to avoid this crash until primitive Self resolution is fixed.
use std::ops::{Index, IndexMut};
use std::slice::SliceIndex;

#[flux_attrs::extern_spec(core::ops)]
#[flux::assoc(fn output_pred(value: Self, index: Idx, out: Self::Output) -> bool { true })]
trait Index<Idx> {}

#[flux_attrs::extern_spec(core::str)]
impl<I: SliceIndex<str>> IndexMut<I> for str {
    #[flux::sig(fn(&mut str[@s], I[@idx]) -> &mut I::Output{out:
        <Self as Index<I>>::output_pred(s, idx, out)})]
    fn index_mut(&mut self, index: I) -> &mut I::Output;
}
