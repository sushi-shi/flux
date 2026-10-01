flux_attrs::defs! {
    fn accepts<T>(predicate: T -> bool, value: T) -> bool { predicate(value) }
}

#[flux::sig(fn[p: bool -> bool](x: i32) requires accepts(p, x))] //~ ERROR mismatched sorts
fn mixed_sorts(x: i32) {}
