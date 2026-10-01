extern crate flux_core;

#[flux_core::refined]
#[flux::invariant(bytes <= 10)]
struct Buffer { bytes: usize }

fn invalid() -> Buffer {
    Buffer { bytes: 11 } //~ ERROR refinement type
}

#[flux_core::ensures(buffer.bytes == old(buffer.bytes))]
fn wrong_frame(buffer: &mut Buffer) { //~ ERROR refinement type
    buffer.bytes = 0;
}
