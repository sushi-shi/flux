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
