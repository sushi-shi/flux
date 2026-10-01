//! Execute the integer model claims against Rust, independently of Flux.

#[test]
fn exhaustive_u8_ceiling_bounds() {
    for value in u8::MIN..=u8::MAX {
        for divisor in 1..=u8::MAX {
            let rounded = u32::from(value.div_ceil(divisor));
            let value = u32::from(value);
            let divisor = u32::from(divisor);
            assert!(rounded * divisor >= value);
            assert!(rounded == 0 || (rounded - 1) * divisor < value);
        }
    }
}

#[test]
fn wide_ceiling_matches_unbounded_numerator_at_boundaries() {
    let values = [0, 1, 31, 32, 33, u64::from(u32::MAX), u64::MAX - 1, u64::MAX];
    let divisors = [1, 2, 31, 32, 33, u64::MAX];
    for value in values {
        for divisor in divisors {
            let expected = (u128::from(value) + u128::from(divisor) - 1) / u128::from(divisor);
            assert_eq!(u128::from(value.div_ceil(divisor)), expected);
        }
    }
    assert_eq!(u128::MAX.div_ceil(2), 1 << 127);
    assert_eq!(u128::MAX.div_ceil(u128::MAX), 1);
}

#[test]
fn widening_preserves_values_and_patch_products() {
    for value in u16::MIN..=u16::MAX {
        assert_eq!(u64::from(value), value as u64);
        assert_eq!(usize::from(value), value as usize);
    }
    let sides = [0, 1, 31, 32, 33, u32::MAX - 1, u32::MAX];
    for width in sides {
        for height in sides {
            let product = u64::from(width) * u64::from(height);
            assert_eq!(u128::from(product), u128::from(width) * u128::from(height));
        }
    }
}
