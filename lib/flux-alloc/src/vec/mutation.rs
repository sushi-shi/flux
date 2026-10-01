//! Conservative mutation models, including a leak-safe bound for draining.

use std::{alloc::Allocator, ops::RangeBounds, vec::Drain};

use flux_attrs::*;

defs! { use flux_core::num::{min, max}; }

// Only the audited byte specialization is enabled. Unknown Clone and Drop
// implementations may run arbitrary user code.
#[assoc(fn byte_copy() -> bool { false })]
#[allow(dead_code)] // Used by refinement expressions.
pub trait ByteElement {}
impl<T> ByteElement for T {}
#[assoc(fn byte_copy() -> bool { true })]
impl ByteElement for u8 {}

#[extern_spec]
impl<T: Clone, A: Allocator> Vec<T, A> {
    // If growth is needed, the old capacity is below the requested final
    // length. Twice that length safely bounds RawVec's doubling policy.
    #[no_panic_if(<A as Allocator>::collection_ops_no_panic()
        && max(8, 2 * (n + m)) <= isize::MAX)]
    #[spec(fn(self: &mut Vec<T, A>[@n], &[T][@m])
        requires <T as ByteElement>::byte_copy()
        ensures self: Vec<T, A>[n + m])]
    fn extend_from_slice(&mut self, other: &[T]);
}

#[extern_spec(core::ops)]
#[assoc(fn modeled_prefix() -> bool { false }
        fn prefix_end(range: Self) -> T)]
trait RangeBounds<T: ?Sized> {}

#[extern_spec(core::ops)]
#[assoc(fn modeled_prefix() -> bool { true }
        fn prefix_end(range: Self) -> T { range.end })]
impl<T> RangeBounds<T> for RangeTo<T> {}

#[extern_spec(std::vec)]
impl<'a, T, A: Allocator> Drain<'a, T, A> {
    // keep_rest can restore drained elements beyond the interval above.
    // Reject it until the owner/iterator effect can be expressed together.
    #[spec(fn(Self) requires false)]
    fn keep_rest(self);
}
