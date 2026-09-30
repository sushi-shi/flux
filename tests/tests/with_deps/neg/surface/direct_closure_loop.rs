extern crate flux_core;
extern crate flux_alloc;

pub fn false_claim(values: &[i32]) {
    let mut output = Vec::new();
    for &value in values {
        let clamp = |n: i32| {
            let result = if n > 0 { n } else { 0 };
            flux_rs::assert(result > 0); //~ ERROR refinement type
            result
        };
        output.push(clamp(value));
    }
}
