use flux_rs::attrs::*;

extern crate flux_core;
use flux_rs::assert;
use std::str::Chars;

// Refining `str` iterators by their remaining length keeps ordinary iteration checkable.

flux_rs::defs! {
    qualifier SzEqA(iter: int, size: int -> int, i: int, iter0: int) { i + size(iter) == size(iter0) }
    qualifier SzEqB(i: int, iter: int, size: int -> int, iter0: int) { i + size(iter) == size(iter0) }
    qualifier SzLeA(iter: int, size: int -> int, i: int, iter0: int) { i + size(iter) <= size(iter0) }
    qualifier SzLeB(i: int, iter: int, size: int -> int, iter0: int) { i + size(iter) <= size(iter0) }
}

#[spec(fn(it: I[@s]) -> usize[<I as Iterator>::size(s)] requires <I as Iterator>::has_size_model())]
pub fn count<I: Iterator>(mut it: I) -> usize {
    let mut n = 0;
    while let Some(_) = it.next() {
        n += 1;
    }
    n
}

#[spec(fn(it: I[@s], upper: usize) requires <I as Iterator>::has_size_model() && <I as Iterator>::size(s) <= upper)]
pub fn loop_enumerate<I: Iterator>(it: I, upper: usize) {
    for (i, _) in it.enumerate() {
        assert(i < upper);
    }
}

#[spec(fn(it: Chars[@s], upper: usize) requires s.size <= upper)]
pub fn test_enumerate_chars(it: Chars, upper: usize) {
    loop_enumerate(it, upper);
}

pub fn test_iterate(s: &str) -> usize {
    let mut total = 0;
    for (_i, c) in s.char_indices() {
        if c == 'a' {
            total += 1;
        }
    }
    for _l in s.lines() {}
    total
}

pub fn reversed_range_has_zero_items() {
    assert(count(5usize..3usize) == 0);
}
