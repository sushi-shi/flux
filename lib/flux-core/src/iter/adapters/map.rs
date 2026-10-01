use flux_attrs::*;

#[extern_spec(core::iter)]
#[refined_by(inner: I)]
struct Map<I, F>;

#[extern_spec(core::iter)]
#[assoc(
    fn has_size_model() -> bool { <I as Iterator>::has_size_model() }
    fn has_fused_model() -> bool { <I as Iterator>::has_fused_model() }
    fn size(x: Map<I>) -> int { <I as Iterator>::size(x.inner) }
    fn done(x: Map<I>) -> bool { <I as Iterator>::done(x.inner) }
    fn step(x: Map<I>, y: Map<I>) -> bool { <I as Iterator>::step(x.inner, y.inner) }
)]
impl<B, I: Iterator, F: FnMut(I::Item) -> B> Iterator for Map<I, F> {
    #[spec(fn(self: &mut Self[@curr_s]) -> Option<B>[!<Self as Iterator>::done(curr_s)]
           requires <I as Iterator>::has_size_model()
           ensures self: Self{next_s: <Self as Iterator>::step(curr_s, next_s)}
           where F: FnMut(I::Item{item: <I as Iterator>::valid_item(curr_s.inner, item)}) -> B)]
    fn next(&mut self) -> Option<B>;
}
