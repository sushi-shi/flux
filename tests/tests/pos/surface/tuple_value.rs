type Pair = (usize, bool);
type Nested = (Pair, char);
type With<T> = (usize, T);

#[flux::sig(fn(pair: Pair[@p]) -> usize[p.0])]
fn first(pair: Pair) -> usize {
    pair.0
}

#[flux::sig(fn(pair: Pair[@p]) -> bool[p.1])]
fn second(pair: Pair) -> bool {
    pair.1
}

#[flux::sig(fn(pair: Nested[@p]) -> char[p.1])]
fn nested(pair: Nested) -> char {
    check(first(pair.0) == pair.0.0);
    pair.1
}

#[flux::sig(fn<T>(pair: With<T>[@p]) -> usize[p.0])]
fn generic<T>(pair: With<T>) -> usize {
    pair.0
}

fn assignment(flag: bool) {
    let mut pair = (4, true);
    check(first(pair) == 4);
    pair.0 = 7;
    check(first(pair) == 7);
    if flag {
        pair.0 = 9;
        pair.1 = false;
    }
    check(first(pair) <= 9);
    check(second(pair) || first(pair) == 9);
}

#[flux::sig(fn(value: &mut usize) ensures value: usize[8])]
fn replace(value: &mut usize) {
    *value = 8;
}

fn borrowed_field() {
    let mut pair = (4, true);
    replace(&mut pair.0);
    check(first(pair) == 8);
}

#[flux::sig(fn<T>(value: T) -> usize[7])]
fn partial_move<T>(value: T) -> usize {
    let pair = (7, value);
    let _moved = pair.1;
    pair.0
}

#[flux::sig(fn(pair: (usize[@n], _)) -> usize[n])]
fn type_hole(pair: (usize, bool)) -> usize {
    pair.0
}

#[flux::sig(fn(limit: usize) -> usize[limit])]
fn count(limit: usize) -> usize {
    let mut pair = (0, false);
    while pair.0 < limit {
        pair.0 += 1;
    }
    first(pair)
}

#[flux::sig(fn(f: F) -> usize where F: FnOnce((usize[0], bool[true])) -> usize)]
fn apply<F: FnOnce(Pair) -> usize>(f: F) -> usize {
    f((0, true))
}

fn callback() {
    apply(|pair| {
        check(pair.0 == 0 && pair.1);
        pair.0
    });
}

fn boxed_field() {
    let mut pair = (7, Box::new(3));
    *pair.1 = 9;
    check(generic(pair) == 7);
}

#[flux::sig(fn(pair: &Pair[@p]) -> usize[p.0])]
fn shared(pair: &Pair) -> usize {
    pair.0
}

#[flux::refined_by(value: int)]
struct Cell {
    #[flux::field(i32[value])]
    value: i32,
}
type RecordPair = (Cell, bool);

#[flux::sig(fn(pair: &RecordPair[@p]) -> i32[p.0.value])]
fn record_value(pair: &RecordPair) -> i32 {
    pair.0.value
}

fn nested_mutation() {
    let mut pair = (Cell { value: 4 }, true);
    pair.0.value = 7;
    check(record_value(&pair) == 7);
}

#[flux::sig(fn(x: bool[true]))]
fn check(x: bool) {}
