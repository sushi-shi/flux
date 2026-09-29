// How general is the arithmetic? (fixpoint + z3; nonlinear is the usual weak spot)
#[flux::sig(fn(n: usize) -> usize{v: v > 25} requires n > 5)]
fn sq(n: usize) -> usize { n * n }                   // nonlinear

#[flux::sig(fn(n: usize) -> usize{v: v % 2 == 0})]
fn dbl(n: usize) -> usize { n + n }                  // linear, modular

#[flux::sig(fn(n: usize) -> usize{v: v % 2 == 0})]
fn prod_consec(n: usize) -> usize { n * (n + 1) }    // true, but needs nonlinear reasoning

// Loop invariant inference (the "liquid" part: no invariant written)
#[flux::sig(fn(n: usize) -> usize[n])]
fn count(n: usize) -> usize {
    let mut i = 0;
    while i < n { i += 1; }
    i
}
fn main() {}
