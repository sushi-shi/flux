trait HasItem { type Item; }
struct Number;
struct Flag;
struct Pair;
impl HasItem for Number { type Item = i32; }
impl HasItem for Flag { type Item = bool; }
impl HasItem for Pair { type Item = (usize, bool); }

#[flux::refined_by(value: <T as HasItem>::Item)]
struct Wrapped<T: HasItem> {
    #[flux::field(<T as HasItem>::Item[value])]
    value: T::Item,
}

#[flux::sig(fn(w: Wrapped<Number>[@s]) -> i32[s.value])]
fn number(w: Wrapped<Number>) -> i32 { w.value }

#[flux::sig(fn(w: Wrapped<Flag>[@s]) -> bool[s.value])]
fn flag(w: Wrapped<Flag>) -> bool { w.value }

#[flux::sig(fn(w: Wrapped<Pair>[@s]) -> usize[s.value.0])]
fn first(w: Wrapped<Pair>) -> usize { w.value.0 }

#[flux::sig(fn(condition: bool[true]))]
fn check(condition: bool) {}

fn instantiate() {
    check(number(Wrapped::<Number> { value: 7 }) == 7);
    check(flag(Wrapped::<Flag> { value: true }));
    check(first(Wrapped::<Pair> { value: (9, false) }) == 9);
}
