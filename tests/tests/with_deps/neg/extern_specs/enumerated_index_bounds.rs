//@compile-flags: -Fstd-extern-specs=on -Fcheck-overflow=strict
use flux_rs::attrs::*;
use flux_rs::assert;
extern crate flux_core;

#[refined_by(phase: int)]
struct Paused {
    #[field(usize[phase])]
    phase: usize,
}
#[assoc(
    fn has_size_model() -> bool { true }
    fn size(self: Self) -> int { if self.phase == 0 || self.phase == 2 { 1 } else { 0 } }
    fn done(self: Self) -> bool { self.phase != 0 && self.phase != 2 }
    fn step(self: Self, next: Self) -> bool {
        next.phase == if self.phase < 3 { self.phase + 1 } else { self.phase }
    }
)]
impl Iterator for Paused {
    type Item = usize;
    #[spec(fn(self: &mut Self[@s]) -> Option<usize>[s.phase == 0 || s.phase == 2]
        ensures self: Self[if s.phase < 3 { s.phase + 1 } else { s.phase }])]
    fn next(&mut self) -> Option<usize> {
        let phase = self.phase;
        if phase < 3 { self.phase += 1; }
        if phase == 0 || phase == 2 { Some(7) } else { None }
    }
}
fn false_counter() {
    let mut iter = Paused { phase: 0 }.enumerate();
    iter.next();
    iter.next();
    if let Some((index, _)) = iter.next() {
        assert(index == 2); //~ ERROR refinement type
    }
}

fn wrong_mapped(xs: &[u8]) {
    xs.iter().enumerate().map(|(index, _)| assert(index >= xs.len())).next(); //~ ERROR refinement type
}

#[refined_by(empty: bool)]
struct Restarts {
    #[field(bool[empty])]
    empty: bool,
}

#[assoc(
    fn has_size_model() -> bool { true }
    fn has_fused_model() -> bool { true }
    fn size(self: Self) -> int { if self.empty { 0 } else { 1 } }
    fn done(self: Self) -> bool { self.empty }
    fn step(self: Self, next: Self) -> bool { next.empty == !self.empty }
)]
impl Iterator for Restarts {
    type Item = usize;
    #[spec(fn(self: &mut Self[@s]) -> Option<usize>[!s.empty] ensures self: Self[!s.empty])]
    fn next(&mut self) -> Option<usize> { //~ ERROR refinement type
        let empty = self.empty;
        self.empty = !empty;
        if empty { None } else { Some(7) }
    }
}
