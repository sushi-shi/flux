//! Runtime falsification checks for the mathematical unsigned multiplication model.
//! Compile with `rustc --test`; these complement, rather than prove, model soundness.

#[test]
fn all_u8_products_match_widened_arithmetic() {
    for a in 0..=u8::MAX {
        for b in 0..=u8::MAX {
            let mathematical = u16::from(a) * u16::from(b);
            let expected = mathematical.min(u16::from(u8::MAX)) as u8;
            assert_eq!(a.saturating_mul(b), expected, "a={a}, b={b}");
        }
    }
}

macro_rules! wider_boundary_tests {
    ($name:ident, $ty:ident) => {
        #[test]
        fn $name() {
            let values: [$ty; 8] =
                [0, 1, 2, 3, $ty::MAX / 4, $ty::MAX / 4 + 1, $ty::MAX - 1, $ty::MAX];
            for a in values {
                for b in values {
                    let mathematical = (a as u128) * (b as u128);
                    let expected = mathematical.min($ty::MAX as u128) as $ty;
                    assert_eq!(a.saturating_mul(b), expected, "a={a}, b={b}");
                }
            }
        }
    };
}
wider_boundary_tests!(u16_boundaries, u16);
wider_boundary_tests!(u32_boundaries, u32);
wider_boundary_tests!(u64_boundaries, u64);
wider_boundary_tests!(usize_boundaries, usize);

#[test]
fn u128_boundaries_without_a_wider_machine_type() {
    assert_eq!(u128::MAX.saturating_mul(0), 0);
    assert_eq!(u128::MAX.saturating_mul(1), u128::MAX);
    assert_eq!(u128::MAX.saturating_mul(2), u128::MAX);
    assert_eq!((u128::MAX / 4).saturating_mul(4), u128::MAX - 3);
    assert_eq!((u128::MAX / 4 + 1).saturating_mul(4), u128::MAX);
}
