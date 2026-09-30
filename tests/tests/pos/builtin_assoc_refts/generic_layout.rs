//! Generic struct layouts must remain symbolic under strict overflow checking.
#![flux::opts(check_overflow = "strict")]

pub struct Tag<T> {
    pub value: T,
    pub text: &'static str,
}

pub fn length<T>(xs: &[Tag<T>]) -> usize {
    xs.len()
}

#[flux::sig(fn(&[Tag<T>][@n]) ensures n * <Tag<T> as Sized>::size_of() <= isize::MAX)]
pub fn layout_bound<T>(_: &[Tag<T>]) {}

pub fn array_length<T, const N: usize>(xs: &[[Tag<T>; N]]) -> usize {
    xs.len()
}
