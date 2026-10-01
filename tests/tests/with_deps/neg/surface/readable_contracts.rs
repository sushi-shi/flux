extern crate flux_core;
use flux_attrs::{ensures, reflect, requires};

#[requires(value < u32::MAX)]
#[ensures(result == value + 1)]
fn increment(value: u32) -> u32 { value + 1 }

fn exceeds_precondition() { increment(u32::MAX); } //~ ERROR refinement type

#[ensures(result == value + 1)]
fn false_postcondition(value: u32) -> u32 { value } //~ ERROR refinement type

#[requires(!values.is_empty())]
fn nonempty(values: &[u8]) {}
fn empty_call() { nonempty(&[]); } //~ ERROR refinement type

#[ensures(width * height <= u32::MAX)]
fn false_patch_grid_lemma(width: u32, height: u32) {} //~ ERROR refinement type

#[reflect]
enum Decision { Allow, Forbidden }
#[ensures(matches!(result, Decision::Forbidden))]
fn weakened_denial() -> Decision { Decision::Allow } //~ ERROR refinement type

#[ensures(result == value)]
#[ensures(result > value)]
fn stacked_conditions_are_all_checked(value: u32) -> u32 { value } //~ ERROR refinement type

#[requires(text == "ab")]
#[ensures(result.starts_with(text))]
fn reversed_prefix(text: &str) -> &str { "a" } //~ ERROR refinement type

#[flux::refined_by(max_dimension: int)]
struct Limits {
    #[flux::field(u32[max_dimension])]
    max_dimension: u32,
}

#[ensures(result == (width <= limits.max_dimension))]
fn ignores_limit(width: u32, limits: Limits) -> bool { true } //~ ERROR refinement type
