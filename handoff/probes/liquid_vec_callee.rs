#[path = "../../tests/tests/lib/rvec.rs"] mod rvec; use rvec::RVec;
// The "liquid" way to say ∀x∈xs. x == 5: refine the ELEMENT type.
// std Vec is only refined by `len` in Flux's extern specs; contents are not modeled.

#[flux::sig(fn(xs: &RVec<i32{v: v == 5}>[@n]) -> i32[5] requires n >= 1)]
fn first_is_five(xs: &RVec<i32>) -> i32 { xs[0] }

#[flux::sig(fn(xs: &RVec<i32{v: v == 5}>[@n]) -> i32[5] requires n % 2 == 1)]
fn odd_first_is_five(xs: &RVec<i32>) -> i32 { xs[0] }

// Caller with a type-level fact: n > 5, all elems == 5.
#[flux::sig(fn(xs: &RVec<i32{v: v == 5}>[@n]) requires n > 5)]
fn caller_ok(xs: &RVec<i32>) {
    first_is_five(xs);          // should PASS: n>5 ⇒ n>=1, elem refinement flows
}

#[flux::sig(fn(xs: &RVec<i32{v: v == 5}>[@n]) requires n > 5)]
fn caller_odd(xs: &RVec<i32>) {
    odd_first_is_five(xs);      // should FAIL: n>5 does not imply n odd
}

#[flux::sig(fn(xs: &RVec<i32{v: v == 5}>[@n]) requires n > 5)]
fn caller_odd_checked(xs: &RVec<i32>) {
    if xs.len() % 2 == 1 {      // should PASS: path condition gives oddness
        odd_first_is_five(xs);
    }
}
fn main() {}
