#![flux::opts(check_overflow = "strict")]
#![flux::no_panic]
extern crate flux_core;

macro_rules! exact_model {
    ($name:ident, $ty:ident) => {
        #[flux_rs::sig(fn(a: $ty, b: $ty) -> $ty[if a * b <= $ty::MAX { a * b } else { $ty::MAX }])]
        pub fn $name(a: $ty, b: $ty) -> $ty {
            a.saturating_mul(b)
        }
    };
}
exact_model!(u8_product, u8);
exact_model!(u16_product, u16);
exact_model!(u32_product, u32);
exact_model!(u64_product, u64);
exact_model!(u128_product, u128);
exact_model!(usize_product, usize);

#[flux_rs::sig(fn(tokens: usize) -> usize[if tokens * 4 <= usize::MAX { tokens * 4 } else { usize::MAX }])]
pub const fn codex_token_budget(tokens: usize) -> usize {
    tokens.saturating_mul(4)
}

pub fn edge_cases() {
    flux_rs::assert(usize::MAX.saturating_mul(0) == 0);
    flux_rs::assert(usize::MAX.saturating_mul(1) == usize::MAX);
    flux_rs::assert(usize::MAX.saturating_mul(4) == usize::MAX);
}

#[flux_rs::sig(fn(value: usize, factor: usize) requires factor >= 1)]
pub fn expansion_lemma(value: usize, factor: usize) {
    let product = value.saturating_mul(factor);
    flux_core::num::lemmas::saturating_mul_expands(value, factor, product);
    flux_rs::assert(value <= product);
}

#[flux_rs::sig(fn(a: usize, b: usize, factor: usize) requires a <= b)]
pub fn monotonicity_lemma(a: usize, b: usize, factor: usize) {
    let product_a = a.saturating_mul(factor);
    let product_b = b.saturating_mul(factor);
    flux_core::num::lemmas::saturating_mul_monotone(a, b, factor, product_a, product_b);
    flux_rs::assert(product_a <= product_b);
}
