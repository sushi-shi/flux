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
    #[ensures(self.active.is_none())]
    fn finish(&mut self) { self.active.take(); }
}

impl<T> State<T> {
    #[ensures(result == self.active.is_some())]
    fn has_value(&self) -> bool { self.active.is_some() }
}

#[may_panic]
#[ensures(result)]
fn after_finish<T>(state: &mut State<T>) -> bool {
    state.finish();
    !state.has_value()
}
