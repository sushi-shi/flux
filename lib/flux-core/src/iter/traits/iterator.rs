use flux_attrs::*;

defs! {
    fn default_iterator_size<T>(self: T) -> int;
    fn default_iterator_done<T>(self: T) -> bool;

    use crate::num::max;
}

#[extern_spec(core::iter)]
#[assoc(
    // Exact remaining-size reasoning requires an explicit model for the impl.
    // In particular, an unrefined iterator must not inherit a decreasing size.
    fn has_size_model() -> bool { false }
    // With a size model, this capability proves permanent exhaustion. It is
    // needed to bound every future enumeration index, including after None.
    fn has_fused_model() -> bool { false }
    // Every future yielded item satisfies this predicate. `next` checks both
    // the returned item and that later states preserve the current guarantee.
    fn valid_item(self: Self, item: Self::Item) -> bool { true }
    fn size(self: Self) -> int { default_iterator_size(self) }
    fn done(self: Self) -> bool { default_iterator_done(self) }
    fn step(self: Self, other: Self) -> bool { true }
)]
trait Iterator {
    #[spec(
        fn(self: &mut Self[@curr_s]) -> Option<Self::Item{item:
            <Self as Iterator>::valid_item(curr_s, item)}>{present:
            <Self as Iterator>::has_size_model() => present == !<Self as Iterator>::done(curr_s)}
        ensures self: Self[#next_s],
                forall item: Self::Item {
                    <Self as Iterator>::valid_item(next_s, item)
                        => <Self as Iterator>::valid_item(curr_s, item)
                },
                <Self as Iterator>::has_size_model() => <Self as Iterator>::step(curr_s, next_s),
                (<Self as Iterator>::has_size_model() && <Self as Iterator>::has_fused_model()
                    && <Self as Iterator>::done(curr_s)) =>
                    (<Self as Iterator>::done(next_s) && <Self as Iterator>::size(next_s) == 0),
                <Self as Iterator>::has_size_model() => if <Self as Iterator>::done(curr_s) {
                    <Self as Iterator>::size(curr_s) == 0
                } else {
                    <Self as Iterator>::size(curr_s) > 0
                        && <Self as Iterator>::size(next_s) == <Self as Iterator>::size(curr_s) - 1
                }
    )]
    fn next(&mut self) -> Option<Self::Item>;

    #[spec(fn(Self[@s]) -> Enumerate<Self>[0, s])]
    fn enumerate(self) -> Enumerate<Self>
    where
        Self: Sized;

    #[no_panic]
    #[spec(fn(Self[@inner]) -> Rev<Self>[inner])]
    fn rev(self) -> Rev<Self>
    where
        Self: Sized + DoubleEndedIterator;

    #[spec(
        fn(Self[@s], f: F) -> Map<Self, F>[s]
        where
            F: FnMut(Self::Item{item: <Self as Iterator>::valid_item(s, item)}) -> B
    )]
    fn map<B, F>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> B;

    #[spec(
        fn[hrn output: B -> bool](Self[@s], F) -> FilterMap<Self, F>[s, |item| output(item)]
        where F: FnMut(Self::Item{item: <Self as Iterator>::valid_item(s, item)}) -> Option<B{item: output(item)}>
    )]
    fn filter_map<B, F>(self, f: F) -> FilterMap<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Option<B>;

    // The selected result is a yielded item for any comparator. Ordering
    // optimality is a separate property; callers here need the item's bounds.
    #[spec(fn(Self[@s], F) -> Option<Self::Item{item: <Self as Iterator>::valid_item(s, item)}>)]
    fn min_by<F>(self, compare: F) -> Option<Self::Item>
    where
        Self: Sized,
        F: FnMut(&Self::Item, &Self::Item) -> core::cmp::Ordering;

    #[spec(fn(Self[@s], n: usize) -> Skip<Self>[max(0, <Self as Iterator>::size(s) - n)] requires <Self as Iterator>::has_size_model())]
    fn skip(self, n: usize) -> Skip<Self>
    where
        Self: Sized;

    #[spec(fn(Self[@s], n: usize) -> Take<Self>[n, s])]
    fn take(self, n: usize) -> Take<Self>
    where
        Self: Sized;

    #[spec(fn(Self[@s], f: F) where F: FnMut(Self::Item{item: <Self as Iterator>::valid_item(s, item)}) -> () )]
    fn for_each<F>(self, f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item);

    #[spec(fn (Self[@s]) -> B{v: <B as FromIterator<Self::Item>>::with_size(v, <Self as Iterator>::size(s))} requires <Self as Iterator>::has_size_model())]
    fn collect<B: FromIterator<Self::Item>>(self) -> B
    where
        Self: Sized;

    /// Core impl: https://github.com/rust-lang/rust/blob/c871d09d1cc32a649f4c5177bb819646260ed120/library/core/src/iter/traits/iterator.rs#L3049
    #[spec(fn(self: &mut Self[@s], P) -> Option<usize{n: n < <Self as Iterator>::size(s)}>
           requires <Self as Iterator>::has_size_model()
           where P: FnMut(Self::Item) -> bool)]
    fn position<P>(&mut self, predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool;
}
