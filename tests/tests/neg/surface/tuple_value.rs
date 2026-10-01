type Pair = (usize, bool);

#[flux::sig(fn(pair: Pair[@p]) -> usize[p.0])]
fn first(pair: Pair) -> usize {
    pair.0
}

fn assignment() {
    let mut pair = (4, true);
    pair.0 = 7;
    check(first(pair) == 4); //~ ERROR refinement type
}

fn branch(flag: bool) {
    let mut pair = (4, true);
    if flag {
        pair.0 = 7;
    }
    check(first(pair) == 4); //~ ERROR refinement type
}

#[flux::sig(fn(value: &mut usize) ensures value: usize[8])]
fn replace(value: &mut usize) {
    *value = 8;
}

fn borrowed_field() {
    let mut pair = (4, true);
    replace(&mut pair.0);
    check(first(pair) == 4); //~ ERROR refinement type
}

// A whole-tuple requirement must be established from the actual fields.
#[flux::sig(fn(pair: Pair{p: p.0 == 0}))]
fn zero(pair: Pair) {}

fn wrong_whole_value() {
    zero((7, true)); //~ ERROR refinement type
}

#[flux::sig(fn(f: F) -> usize where F: FnOnce((usize[0], bool[true])) -> usize)]
fn apply<F: FnOnce(Pair) -> usize>(f: F) -> usize {
    f((0, true))
}

fn callback() {
    apply(|pair| {
        check(pair.0 == 1); //~ ERROR refinement type
        pair.0
    });
}

#[flux::sig(fn(pair: &mut Pair[@p]) -> usize[p.0])]
fn weak_mutation(pair: &mut Pair) -> usize {
    pair.0 = 7;
    pair.0 //~ ERROR refinement type
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
    check(record_value(&pair) == 4); //~ ERROR refinement type
}

#[flux::sig(fn(x: bool[true]))]
fn check(x: bool) {}
