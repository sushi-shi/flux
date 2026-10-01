flux_attrs::defs! {
    fn accepts<T>(predicate: T -> bool, value: T) -> bool { predicate(value) }
}

#[flux::opaque]
#[flux::refined_by(predicate: int -> bool)]
struct IntPredicate;
#[flux::opaque]
#[flux::refined_by(predicate: bool -> bool)]
struct BoolPredicate;

#[flux::sig(fn(IntPredicate[@p], x: i32{v: accepts(p, v)}) -> i32{v: p(v)})]
fn positive(_: IntPredicate, x: i32) -> i32 {
    x
}

#[flux::sig(fn(BoolPredicate[@p], x: bool{v: accepts(p, v)}) -> bool{v: p(v)})]
fn boolean(_: BoolPredicate, x: bool) -> bool {
    x
}
