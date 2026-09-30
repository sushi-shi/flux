use flux_rs::attrs::*;

extern crate flux_core;
use flux_rs::assert;

// Loop bodies over iterators must stay reachable, and facts about how many items an iterator
// yields must not be invented. Every error below used to be missed: `Map` handed out items whose
// refinement fixpoint solved to `false`, and `str` iterators had no refinement, so the `Iterator`
// contract ("each `Some` shrinks `size`") was contradictory for them.

// --- enumerate over `str` iterators ---

pub fn test_enumerate_chars(s: &str) {
    for (_i, _c) in s.chars().enumerate() {
        assert(false); //~ ERROR refinement type error
    }
}

pub fn test_enumerate_lines(s: &str) {
    for (_i, _l) in s.lines().enumerate() {
        assert(false); //~ ERROR refinement type error
    }
}

#[spec(fn(&str) -> usize[0])]
pub fn test_count_is_zero(s: &str) -> usize {
    let mut n = 0;
    for _ in s.chars().enumerate() {
        n += 1;
    }
    n //~ ERROR refinement type error
}

// --- map ---

pub fn test_map_chars(s: &str) {
    for _c in s.chars().map(|c| c) {
        assert(false); //~ ERROR refinement type error
    }
}

pub fn test_map_slice(xs: &[u8]) {
    for _x in xs.iter().map(|x| x) {
        assert(false); //~ ERROR refinement type error
    }
}

// --- generic code instantiated with a `str` iterator ---

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

pub fn test_count_chars() {
    assert(count("ab".chars()) == count("abc".chars())); //~ ERROR refinement type error
}
