use flux_attrs::*;

defs! {
    fn accepts<T>(predicate: T -> bool, item: T) -> bool { predicate(item) }
}

// Keep the initial inner state as the callback's input envelope. Iteration
// may skip several items, each covered by that state's valid_item predicate.
#[extern_spec(core::iter)]
#[refined_by(origin: I, hrn output: <<F as FnOnce<(<I as Iterator>::Item,)>>::Output as IntoIterator>::Item -> bool)]
struct FilterMap<I, F>;

#[extern_spec(core::iter)]
#[assoc(fn valid_item(s: Self, item: B) -> bool {
    accepts(s.output, item)
})]
impl<B, I: Iterator, F: FnMut(I::Item) -> Option<B>> Iterator for FilterMap<I, F> {
    #[spec(fn(self: &mut Self[@origin, @output]) -> Option<B{item: output(item)}>
        ensures self: Self[origin, output]
        where F: FnMut(I::Item{item: <I as Iterator>::valid_item(origin, item)}) -> Option<B{item: output(item)}>)]
    fn next(&mut self) -> Option<B>;
}
