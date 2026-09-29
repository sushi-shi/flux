// Soundness probes for iterator reasoning. Run with the std specs on:
//   cargo x run handoff/probes/soundness_iterators.rs -- -Fstd-extern-specs
// Every function below states something FALSE; a sound checker rejects each one.
// Status on branch `handoff` (= PR #1 + notes) is in handoff/HANDOFF.md.
extern crate flux_core;
use flux_rs::attrs::*;

flux_rs::defs! {
    qualifier SzEqA(iter: int, size: int -> int, i: int, iter0: int) { i + size(iter) == size(iter0) }
    qualifier SzEqB(i: int, iter: int, size: int -> int, iter0: int) { i + size(iter) == size(iter0) }
}

// Fixed by PR #1 (str iterators refined by remaining length).
#[spec(fn(&str) -> bool[true] ensures 1 == 2)]
pub fn enumerate_chars_falsity(s: &str) -> bool { for (_i, _c) in s.chars().enumerate() { return true; } loop {} }

// Fixed by PR #1 (checker now checks the impl's `F: FnMut(..) -> B` clause at `Map::next`).
#[spec(fn(&[u8]) -> i32[0])]
pub fn map_body_dead(xs: &[u8]) -> i32 { for _ in xs.iter().map(|x| x) { return 1; } 0 }

#[spec(fn(it: I[@s]) -> usize[<I as Iterator>::size(s)])]
pub fn count<I: Iterator>(mut it: I) -> usize { let mut n = 0; while let Some(_) = it.next() { n += 1; } n }

// Fixed by PR #1.
pub fn count_chars_equal() { flux_rs::assert(count("ab".chars()) == count("abc".chars())); }

// STILL OPEN: `Filter` has no refinement, so the Iterator contract is contradictory for it.
pub fn count_filter_equal(a: &[u8], b: &[u8]) {
    flux_rs::assert(count(a.iter().filter(|_| true)) == count(b.iter().filter(|_| true)));
}
