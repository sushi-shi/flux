trait HasItem { type Item; }
struct Number;
struct Flag;
impl HasItem for Number { type Item = i32; }
impl HasItem for Flag { type Item = bool; }

#[flux::refined_by(value: <T as HasItem>::Item)]
struct Wrapped<T: HasItem> {
    #[flux::field(<T as HasItem>::Item[value])]
    value: T::Item,
}

#[flux::sig(fn(w: Wrapped<Number>[@s]) -> i32[s.value])]
fn wrong_number(w: Wrapped<Number>) -> i32 {
    w.value + 1 //~ ERROR refinement type
}

#[flux::sig(fn(w: Wrapped<Flag>[@s]) -> bool[s.value])]
fn wrong_flag(w: Wrapped<Flag>) -> bool {
    !w.value //~ ERROR refinement type
}
