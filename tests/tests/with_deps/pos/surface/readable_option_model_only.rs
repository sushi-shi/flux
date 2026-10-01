//@compile-flags: -Fstd-extern-specs=on
// No ordinary core path or generated field-method witness to force core loading.
use flux_core::refined;

#[refined(active)]
struct State { active: Option<u32> }

fn empty() -> State { State { active: None } }
