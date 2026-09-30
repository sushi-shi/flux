#[path = "../../tests/tests/lib/rvec.rs"] mod rvec; use rvec::RVec;
#[flux::trusted]
#[flux::sig(fn() -> RVec<i32{v: v == 5}>[0])]
fn empty5() -> RVec<i32> { RVec::new() }
#[flux::sig(fn(x: i32) -> Option<RVec<i32{v: v == 5}>>)]
fn no_loop(x: i32) -> Option<RVec<i32>> {
    let mut out = empty5();
    if x != 5 { return None; }
    out.push(x);
    Some(out)
}
#[flux::sig(fn(xs: &RVec<i32>) -> RVec<i32{v: v == 5}>)]
fn with_loop(xs: &RVec<i32>) -> RVec<i32> {
    let mut out = empty5();
    let mut i = 0;
    while i < xs.len() {
        let x = xs[i];
        if x == 5 { out.push(x); }
        i += 1;
    }
    out
}
fn main() {}
