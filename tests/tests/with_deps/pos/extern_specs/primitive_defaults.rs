//@compile-flags: -Fno-panic=on
extern crate flux_core;

#[derive(Default)]
#[flux::refined_by(bytes: int)]
#[flux::invariant(bytes <= 1048576)]
struct Buffer {
    #[flux::field(usize[bytes])]
    bytes: usize,
}

fn primitives() {
    assert!(u8::default() == 0 && u16::default() == 0 && u32::default() == 0);
    assert!(u64::default() == 0 && u128::default() == 0 && usize::default() == 0);
    assert!(i8::default() == 0 && i16::default() == 0 && i32::default() == 0);
    assert!(i64::default() == 0 && i128::default() == 0 && isize::default() == 0);
    assert!(!bool::default());
}
