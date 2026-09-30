extern crate flux_core;
extern crate flux_alloc;

#[flux::sig(fn(&[i32]) -> Vec<i32{v: v >= 0}>)]
pub fn clamp_values(values: &[i32]) -> Vec<i32> {
    let mut output = Vec::new();
    for &value in values {
        let clamp = |n: i32| {
            let result = if n > 0 { n } else { 0 };
            flux_rs::assert(result >= 0);
            result
        };
        output.push(clamp(value));
    }
    output
}
