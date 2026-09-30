#![flux::opts(check_overflow = "strict")]
extern crate flux_core;

pub fn saturation_is_not_wrapping() {
    flux_rs::assert(255u8.saturating_mul(2) == 254); //~ ERROR refinement type
}

pub fn saturation_is_not_unbounded() {
    flux_rs::assert(200u8.saturating_mul(2) < 255); //~ ERROR refinement type
}

pub fn zero_does_not_saturate() {
    flux_rs::assert(u64::MAX.saturating_mul(0) == u64::MAX); //~ ERROR refinement type
}

#[flux_rs::sig(fn(tokens: usize) -> usize[if tokens * 4 <= usize::MAX { tokens * 4 } else { usize::MAX }])]
pub fn wrong_token_multiplier(tokens: usize) -> usize {
    tokens.saturating_mul(3) //~ ERROR refinement type
}

pub fn cannot_apply_expansion_to_zero_factor() {
    flux_core::num::lemmas::saturating_mul_expands(5, 0, 0); //~ ERROR refinement type
}

pub fn monotonicity_is_not_strict() {
    let a = usize::MAX - 1;
    let b = usize::MAX;
    let product_a = a.saturating_mul(4);
    let product_b = b.saturating_mul(4);
    flux_core::num::lemmas::saturating_mul_monotone(a, b, 4, product_a, product_b);
    flux_rs::assert(product_a < product_b); //~ ERROR refinement type
}

#[flux_rs::sig(fn(value: usize, factor: usize, product: usize)
    requires factor >= 1 && product == (if value * factor <= usize::MAX { value * factor } else { usize::MAX })
    ensures value < product)]
pub const fn false_strict_expansion_lemma(_value: usize, _factor: usize, _product: usize) {} //~ ERROR refinement type
