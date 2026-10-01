//@compile-flags: -Fstd-extern-specs=on -Fcheck-overflow=strict -Fno-panic=on
use flux_core::{ensures, may_panic, refined};

#[refined(active)]
struct State<T> { active: Option<T> }

trait Finish {
    #[may_panic]
    fn finish(&mut self);
}

impl<T> Finish for State<T> {
    #[may_panic]
    #[ensures(self.active.is_none())] //~ ERROR refinement type
    fn finish(&mut self) {}
}

#[ensures(result.is_char_boundary(1))]
fn middle_of_codepoint() -> &'static str { "é" } //~ ERROR refinement type
//~| ERROR refinement type
