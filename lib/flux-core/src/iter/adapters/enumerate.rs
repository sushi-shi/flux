use flux_attrs::*;

#[extern_spec(core::iter)]
#[refined_by(idx: int, inner: I)]
#[invariant(0 <= idx && idx <= usize::MAX)]
struct Enumerate<I>;

#[extern_spec(core::iter)]
#[assoc(
    fn has_size_model() -> bool { <I as Iterator>::has_size_model() }
    fn has_fused_model() -> bool { <I as Iterator>::has_fused_model() }
    fn valid_item(x: Enumerate<I>, item: (int, <I as Iterator>::Item)) -> bool {
        (<I as Iterator>::has_size_model() && <I as Iterator>::has_fused_model()
            && 0 <= x.idx && 0 <= <I as Iterator>::size(x.inner)
            && x.idx + <I as Iterator>::size(x.inner) <= usize::MAX) =>
            (x.idx <= item.0 && item.0 < x.idx + <I as Iterator>::size(x.inner))
    }
    fn size(x: Enumerate<I>) -> int { <I as Iterator>::size(x.inner) }
    fn done(x: Enumerate<I>) -> bool { <I as Iterator>::done(x.inner) }
    fn step(x: Enumerate<I>, y: Enumerate<I>) -> bool {
        y.idx == if <I as Iterator>::done(x.inner) {
            x.idx
        } else if x.idx == usize::MAX {
            0
        } else {
            x.idx + 1
        } && <I as Iterator>::step(x.inner, y.inner)
    }
)]
impl<I: Iterator> Iterator for Enumerate<I> {
    #[spec(fn(self: &mut Enumerate<I>[@curr_s]) -> Option<(usize[curr_s.idx], _)>[!<I as Iterator>::done(curr_s.inner)]
           requires <I as Iterator>::has_size_model()
           ensures self: Enumerate<I>[#next_s],
                   <Self as Iterator>::step(curr_s, next_s),
                   (<I as Iterator>::has_fused_model() && <I as Iterator>::done(curr_s.inner)) =>
                       (<I as Iterator>::done(next_s.inner) && <I as Iterator>::size(next_s.inner) == 0),
                   if <I as Iterator>::done(curr_s.inner) {
                       <I as Iterator>::size(curr_s.inner) == 0
                   } else {
                       <I as Iterator>::size(curr_s.inner) > 0
                           && <I as Iterator>::size(next_s.inner) == <I as Iterator>::size(curr_s.inner) - 1
                   })]
    fn next(&mut self) -> Option<(usize, <I as Iterator>::Item)>;
}
