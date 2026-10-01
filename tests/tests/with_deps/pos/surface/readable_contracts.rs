//@compile-flags: -Fcheck-overflow=strict -Fno-panic=on
extern crate flux_core;
use flux_attrs::{ensures, reflect, requires};

#[requires(value < u32::MAX)]
#[ensures(result == value + 1)]
fn increment(value: u32) -> u32 { value + 1 }

#[ensures(result >= value)]
#[requires(value < u32::MAX)]
fn reordered(value: u32) -> u32 { increment(value) }

#[requires(!values.is_empty())]
#[ensures(result == values.len())]
fn length(values: &[u8]) -> usize { values.len() }

#[ensures(result == text.len())]
fn bytes(text: &str) -> usize { text.len() }

#[ensures(result == text)]
fn identity(text: &str) -> &str { text }

#[ensures(width * height <= u64::MAX)]
fn patch_grid_product_fits_u64(width: u32, height: u32) {}

#[reflect]
enum Decision { Allow, Prompt, Forbidden }

#[requires(matches!(decision, Decision::Prompt | Decision::Forbidden))]
#[ensures(matches!(result, Decision::Prompt | Decision::Forbidden))]
fn preserve_restriction(decision: Decision) -> Decision { decision }

#[ensures(matches!(result, Decision::Forbidden))]
fn deny() -> Decision { Decision::Forbidden }

#[requires(text == "ab")]
#[ensures(text.starts_with(result))]
fn prefix(text: &str) -> &str { "a" }

#[requires(text == "ab")]
#[ensures(text.ends_with(result))]
fn suffix(text: &str) -> &str { "b" }
