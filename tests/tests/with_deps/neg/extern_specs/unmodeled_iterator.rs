extern crate flux_core;
use flux_rs::attrs::*;

flux_rs::defs! {
    qualifier SzEqA(iter: int, size: int -> int, i: int, iter0: int) { i + size(iter) == size(iter0) }
    qualifier SzEqB(i: int, iter: int, size: int -> int, iter0: int) { i + size(iter) == size(iter0) }
}

#[spec(fn(it: I[@s]) -> usize[<I as Iterator>::size(s)] requires <I as Iterator>::has_size_model())]
pub fn count<I: Iterator>(mut it: I) -> usize {
    let mut n = 0;
    while let Some(_) = it.next() {
        n += 1;
    }
    n
}

pub fn count_filter_equal(a: &[u8], b: &[u8]) {
    let _a = count(a.iter().filter(|_| true)); //~ ERROR refinement type
    let _b = count(b.iter().filter(|_| true)); //~ ERROR refinement type
}

pub fn infinite_iterator_has_no_size_model() {
    let _ = count(std::iter::repeat(7)); //~ ERROR refinement type
}

pub fn reversed_range_cannot_prove_false() {
    flux_rs::assert(count(5usize..3usize) == 1); //~ ERROR refinement type
}
