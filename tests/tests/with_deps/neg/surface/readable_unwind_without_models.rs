// No -Fstd-extern-specs: the unwind boundary must be guarded by the checker.
use flux_core::{refined, may_panic, ensures};
#[refined]
#[flux::invariant(count <= 3)]
struct State { count: usize }

#[may_panic]
fn corrupt(state: &mut State) { state.count = 10; panic!("stop"); }

#[may_panic]
#[ensures(result <= 3)]
fn caught() -> usize {
    let mut state = State { count: 0 };
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| corrupt(&mut state))); //~ ERROR refinement type
    state.count
}

fn indirect_catch(f: fn()) {
    let catch: fn(fn()) -> std::thread::Result<()> = std::panic::catch_unwind; //~ ERROR refinement type
    let _ = catch(f);
}
