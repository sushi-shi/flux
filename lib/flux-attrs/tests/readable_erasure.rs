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
