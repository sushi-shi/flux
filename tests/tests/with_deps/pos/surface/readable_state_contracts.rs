//@compile-flags: -Fcheck-overflow=strict -Fno-panic=on
extern crate flux_core;
extern crate flux_alloc;
use flux_attrs::{ensures, requires};

#[flux::refined_by(next: int, bytes: int)]
struct State {
    #[flux::field(u32[next])]
    next: u32,
    #[flux::field(usize[bytes])]
    bytes: usize,
}

impl State {
    #[ensures(result == self.next)]
    fn current(&self) -> u32 { self.next }

    #[ensures(result == old(self.next))]
    #[ensures(self.next == if old(self.next) < u32::MAX { old(self.next) + 1 } else { u32::MAX })]
    #[ensures(self.bytes == old(self.bytes))]
    fn reserve(&mut self) -> u32 {
        let previous = self.next;
        self.next = self.next.saturating_add(1);
        previous
    }

    #[ensures(result.is_ok() == (old(self.next) < u32::MAX))]
    #[ensures(self.next == if result.is_ok() { old(self.next) + 1 } else { old(self.next) })]
    #[ensures(self.bytes == old(self.bytes))]
    fn advance(&mut self) -> Result<(), ()> {
        self.next = self.next.checked_add(1).ok_or(())?;
        Ok(())
    }
}

#[requires(state.next < u32::MAX)]
#[ensures(state.next == old(state.next) + 1)]
#[ensures(state.bytes == old(state.bytes))]
fn advance_caller(state: &mut State) {
    let before = state.current();
    let bytes = state.bytes;
    let outcome = state.advance();
    assert!(outcome.is_ok());
    assert!(state.next == before + 1);
    assert!(state.bytes == bytes);
}

fn exhaustion_caller() {
    let mut state = State { next: u32::MAX, bytes: 7 };
    assert!(state.advance().is_err());
    assert!(state.next == u32::MAX);
    assert!(state.reserve() == u32::MAX);
    assert!(state.next == u32::MAX);
    assert!(state.bytes == 7);
}

#[ensures(left.next == old(right.next))]
#[ensures(right.next == old(left.next))]
#[ensures(left.bytes == old(left.bytes))]
#[ensures(right.bytes == old(right.bytes))]
fn exchange(left: &mut State, right: &mut State) {
    let saved = left.next;
    left.next = right.next;
    right.next = saved;
}

#[ensures(result.is_ok())]
fn nested_result() -> Result<Vec<Vec<u8>>, ()> { Ok(Vec::new()) }

#[requires(!values.is_empty())]
#[ensures(result == values.len())]
fn vector_length(values: Vec<u8>) -> usize { values.len() }
