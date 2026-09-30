//! Checked arithmetic facts for callers of the numeric models.
//!
//! These functions have no runtime work. Their postconditions are proved from
//! their parameter refinements; none is marked trusted. The module is available
//! when Flux models are enabled, so production call sites can use `#[cfg(flux)]`.

#![cfg_attr(flux, flux::opts(check_overflow = "strict"))]

use flux_attrs::spec;

/// Scaling an unsigned budget by a positive factor cannot shrink it, even when
/// the product saturates. A zero factor is deliberately excluded.
#[spec(fn(value: usize, factor: usize, product: usize)
       requires factor >= 1 && product == super::min(value * factor, usize::MAX)
       ensures value <= product)]
pub const fn saturating_mul_expands(_value: usize, _factor: usize, _product: usize) {}

/// Increasing a budget cannot decrease its saturated product. Saturation can
/// erase a strict inequality, so this lemma only promises a non-strict one.
#[spec(fn(a: usize, b: usize, factor: usize, product_a: usize, product_b: usize)
       requires a <= b
           && product_a == super::min(a * factor, usize::MAX)
           && product_b == super::min(b * factor, usize::MAX)
       ensures product_a <= product_b)]
pub const fn saturating_mul_monotone(
    _a: usize,
    _b: usize,
    _factor: usize,
    _product_a: usize,
    _product_b: usize,
) {
}
