//@compile-flags: -Fcheck-overflow=strict -Fno-panic=on
extern crate flux_core;
use flux_attrs::{ensures, requires};

#[flux::refined_by(next: int, bytes: int)]
struct State {
    #[flux::field(u32[next])]
    next: u32,
    #[flux::field(usize[bytes])]
    bytes: usize,
}

impl State {
    #[ensures(self.next == old(self.next))]
    fn false_frame(&mut self) { self.next = 0; } //~ ERROR refinement type

    #[ensures(result == old(self.next))]
    fn returns_new_state(&mut self) -> u32 {
        self.next = self.next.saturating_add(1);
        self.next //~ ERROR refinement type
    }

    #[ensures(result.is_ok() == (old(self.next) < u32::MAX))]
    fn false_success(&mut self) -> Result<(), ()> { Ok(()) } //~ ERROR refinement type

    #[requires(self.next < u32::MAX)]
    #[ensures(self.next == old(self.next) + 1)]
    fn advance(&mut self) { self.next += 1; }

    #[ensures(result.is_err())]
    #[ensures(self.next == old(self.next))]
    fn failure_changes_state(&mut self) -> Result<(), ()> {
        self.next = 0;
        Err(()) //~ ERROR refinement type
    }
}

fn requires_entry_state() {
    let mut state = State { next: u32::MAX, bytes: 0 };
    state.advance(); //~ ERROR refinement type
}

fn omitted_frame_is_not_assumed() {
    let mut state = State { next: 0, bytes: 7 };
    state.advance();
    assert!(state.bytes == 7); //~ ERROR may panic
}
