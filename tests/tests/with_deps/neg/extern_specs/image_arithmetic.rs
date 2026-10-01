extern crate flux_core;
use flux_rs::assert;

fn zero_divisor(n: u32) { n.div_ceil(0); } //~ ERROR refinement type
fn unknown_divisor(n: u32, d: u32) { n.div_ceil(d); } //~ ERROR refinement type
fn wrong_rounding() { assert(33u32.div_ceil(32) == 1); } //~ ERROR refinement type
fn wrapped_numerator() { assert(u32::MAX.div_ceil(32) == 0); } //~ ERROR refinement type
#[flux::sig(fn(n: u32) -> u64[n + 1])]
fn wrong_widening(n: u32) -> u64 { u64::from(n) } //~ ERROR refinement type
