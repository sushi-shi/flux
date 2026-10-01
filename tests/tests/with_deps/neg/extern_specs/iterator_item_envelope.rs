//@compile-flags: -Fstd-extern-specs=on
use flux_rs::attrs::*;
use flux_rs::assert;
extern crate flux_core;

#[refined_by(remaining: int)]
#[invariant(remaining >= 0)]
struct One {
    #[field(usize[remaining])]
    remaining: usize,
}
#[assoc(
    fn has_size_model() -> bool { true }
    fn valid_item(self: Self, item: int) -> bool { false }
    fn size(self: Self) -> int { if self.remaining > 0 { self.remaining } else { 0 } }
    fn done(self: Self) -> bool { self.remaining <= 0 }
    fn step(before: Self, after: Self) -> bool {
        after.remaining == if before.remaining > 0 { before.remaining - 1 } else { 0 }
    }
)]
impl Iterator for One {
    type Item = usize;
    #[spec(fn(self: &mut Self[@before]) -> Option<usize>[before.remaining > 0]
        ensures self: Self[if before.remaining > 0 { before.remaining - 1 } else { 0 }])]
    fn next(&mut self) -> Option<usize> { //~ ERROR refinement type error
        if self.remaining > 0 {
            self.remaining -= 1;
            Some(7)
        } else { None }
    }
}
fn false_claim() {
    for _ in (One { remaining: 1 }).map(|x| x) {
        assert(false);
    }
}

// Associated Item sorts must match the concrete bool/char arguments used by
// the solver, and an unknown item's predicate must not erase a reachable arm.
fn generic_bool<I: Iterator<Item = bool>>(mut iter: I) {
    if let Some(_) = iter.next() {
        assert(false); //~ ERROR refinement type error
    }
}

fn generic_char<I: Iterator<Item = char>>(mut iter: I) {
    if let Some(_) = iter.next() {
        assert(false); //~ ERROR refinement type error
    }
}

// Size support is independent of the obligation to justify item facts.
struct NoSize { pending: bool }
#[assoc(fn valid_item(self: Self, item: int) -> bool { false })]
impl Iterator for NoSize {
    type Item = usize;
    #[spec(fn(self: &mut Self) -> Option<usize> ensures self: Self)]
    fn next(&mut self) -> Option<usize> { //~ ERROR refinement type error
        if self.pending { self.pending = false; Some(7) } else { None }
    }
}

// Establishing a property only for the next item is insufficient for for_each:
// later items must also satisfy the original iterator's item predicate.
#[refined_by(remaining: int)]
struct Countdown {
    #[field(usize[remaining])]
    remaining: usize,
}
#[assoc(fn valid_item(self: Self, item: int) -> bool { item == self.remaining })]
impl Iterator for Countdown {
    type Item = usize;
    #[spec(fn(self: &mut Self[@before]) -> Option<usize[before.remaining]>
        ensures self: Self[if before.remaining > 0 { before.remaining - 1 } else { 0 }])]
    fn next(&mut self) -> Option<usize> { //~ ERROR refinement type error
        if self.remaining > 0 {
            let item = self.remaining;
            self.remaining -= 1;
            Some(item)
        } else { None }
    }
}
