use flux_attrs::*;

#[extern_spec(core::iter)]
#[assoc(
    fn modeled_back() -> bool { false }
    fn back_no_panic() -> bool { false }
    fn back_step(before: Self, after: Self) -> bool { true }
)]
trait DoubleEndedIterator: Iterator {
    #[no_panic_if(<Self as DoubleEndedIterator>::back_no_panic())]
    #[spec(fn(self: &mut Self[@before]) -> Option<Self::Item{item:
        <Self as Iterator>::valid_item(before, item)}>
        requires <Self as DoubleEndedIterator>::modeled_back()
        ensures self: Self{after: <Self as DoubleEndedIterator>::back_step(before, after)})]
    fn next_back(&mut self) -> Option<Self::Item>;
}
