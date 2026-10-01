extern crate flux_core;

#[flux::sig(fn() -> usize[1])]
fn wrong_default() -> usize { usize::default() } //~ ERROR refinement type

struct Custom { value: usize }
impl Default for Custom {
    fn default() -> Self { Self { value: 7 } }
}

#[flux::sig(fn() -> usize[0])]
fn arbitrary_defaults_are_not_zero() -> usize { Custom::default().value } //~ ERROR refinement type
