//@compile-flags: -Fstd-extern-specs=on
use flux_core::{refined, may_panic, ensures};
#[refined]
#[flux::invariant(count <= 3)]
struct State { count: usize }

#[may_panic]
fn corrupt(state: &mut State) { state.count = 10; panic!("stop"); }

// This returns 10 at runtime. Before modeling the unsupported recovery boundary,
// the checker accepted the false <= 3 contract by retaining the old invariant.
#[may_panic]
#[ensures(result <= 3)]
fn caught() -> usize {
    let mut state = State { count: 0 };
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| corrupt(&mut state))); //~ ERROR refinement type
    state.count
}
