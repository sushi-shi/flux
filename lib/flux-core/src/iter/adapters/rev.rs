use flux_attrs::*;

#[extern_spec(core::iter)]
#[refined_by(inner: T)]
struct Rev<T>;

#[extern_spec(core::iter)]
impl<I: DoubleEndedIterator> Iterator for Rev<I> {
    #[no_panic_if(<I as DoubleEndedIterator>::back_no_panic())]
    #[spec(fn(self: &mut Self[@before]) -> Option<I::Item{item:
        <I as Iterator>::valid_item(before.inner, item)}>
        requires <I as DoubleEndedIterator>::modeled_back()
        ensures self: Self{after:
            <I as DoubleEndedIterator>::back_step(before.inner, after.inner)})]
    fn next(&mut self) -> Option<I::Item>;
}
