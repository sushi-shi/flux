#![flux::opts(check_overflow = "strict")]

pub struct Tag<T> {
    pub value: T,
}

#[flux::sig(fn() ensures <Tag<T> as Sized>::size_of() == 0)]
pub fn unknown_size_is_not_zero<T>() {} //~ ERROR refinement type

#[flux::sig(fn() ensures <Tag<T> as Sized>::align_of() == 1)]
pub fn unknown_alignment_is_not_one<T>() {} //~ ERROR refinement type

#[flux::sig(fn(&[Tag<T>][@n]) ensures n == 0)]
pub fn generic_slice_is_not_empty<T>(_: &[Tag<T>]) {} //~ ERROR refinement type
