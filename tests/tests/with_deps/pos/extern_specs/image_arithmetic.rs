//@compile-flags: -Fcheck-overflow=strict -Fno-panic=on
extern crate flux_core;
use flux_rs::assert;

#[flux::sig(fn(value: u32) -> u64[value])]
fn widen(value: u32) -> u64 { u64::from(value) }

#[flux::sig(fn(value: u32) -> u32[(value + 31) / 32])]
fn patches(value: u32) -> u32 { value.div_ceil(32) }

#[flux::sig(fn(value: u32, divisor: u32) -> u32[(value + divisor - 1) / divisor] requires divisor > 0)]
fn variable_divisor(value: u32, divisor: u32) -> u32 { value.div_ceil(divisor) }

fn boundaries() {
    assert(0u8.div_ceil(255) == 0);
    assert(255u8.div_ceil(2) == 128);
    assert(u16::MAX.div_ceil(2) == 32768);
    assert(u32::MAX.div_ceil(32) == 134217728);
    assert(u64::MAX.div_ceil(2) == 9223372036854775808);
    assert(u128::MAX.div_ceil(2) == 170141183460469231731687303715884105728);
    assert(usize::MAX.div_ceil(usize::MAX) == 1);
    assert(u64::from(u32::MAX) == 4294967295);
    assert(u128::from(u64::MAX) == 18446744073709551615);
}

#[flux::sig(fn(width: u32, height: u32) -> u64[width * height])]
fn product(width: u32, height: u32) -> u64 {
    u64::from(width) * u64::from(height)
}
