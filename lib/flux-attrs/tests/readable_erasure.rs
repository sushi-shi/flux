use flux_attrs::{ensures, requires};

#[requires(value > 0)]
#[ensures(result == value + 1)]
fn unchanged(value: u32) -> u32 {
    value
}

#[test]
fn contracts_have_no_runtime_checks_or_effects() {
    assert_eq!(unchanged(0), 0);
    assert_eq!(unchanged(u32::MAX), u32::MAX);
}

// An unchecked dependency may use syntax outside the checker's current subset.
// Even when Cargo builds its proc-macros for verification, that dependency must
// receive its original Rust item, without a generated compile_error or witness.
#[ensures(unsupported_predicate(result))]
fn unchecked_dependency(value: u32) -> u32 {
    value.saturating_add(1)
}

#[test]
fn unsupported_conditions_are_erased_in_unchecked_crates() {
    assert_eq!(unchecked_dependency(7), 8);
    assert_eq!(unchecked_dependency(u32::MAX), u32::MAX);
}

#[flux_attrs::refined]
struct Counter {
    next: u32,
}

impl Counter {
    #[ensures(self.next == old(self.next) + 100)]
    #[ensures(result.is_err())]
    fn advance(&mut self) -> Result<(), ()> {
        self.next += 1;
        Ok(())
    }
}

#[test]
fn state_contracts_do_not_execute_old_or_postconditions() {
    let mut counter = Counter { next: 0 };
    assert_eq!(counter.advance(), Ok(()));
    assert_eq!(counter.next, 1);
}

trait Flush {
    #[flux_attrs::may_panic]
    fn flush(&mut self);
}

#[flux_attrs::refined(active)]
struct OptionalState<T> {
    active: Option<T>,
}

impl<T> Flush for OptionalState<T> {
    #[flux_attrs::may_panic]
    #[ensures(self.active.is_none())]
    fn flush(&mut self) {
        self.active.take();
    }
}

#[test]
fn optional_state_and_trait_effects_erase_in_native_builds() {
    let mut state = OptionalState { active: Some(String::from("payload")) };
    let erased: &mut dyn Flush = &mut state;
    erased.flush();
    assert_eq!(state.active, None);
}
