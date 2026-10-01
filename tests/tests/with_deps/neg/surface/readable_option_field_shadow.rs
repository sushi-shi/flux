//@compile-flags: -Fstd-extern-specs=on
use flux_core::{ensures, refined};

#[refined]
struct Impostor { is_some: bool }
impl Impostor { fn is_none(&self) -> bool { true } }
#[refined]
struct Holder { active: Impostor }
#[ensures(result == holder.active.is_none())] //~ ERROR trait bound
fn fake(holder: &Holder) -> bool { holder.active.is_none() }
