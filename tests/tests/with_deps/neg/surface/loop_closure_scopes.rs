extern crate flux_core;
extern crate flux_alloc;

pub fn false_claim_in_loop_closure(values: &[i32]) {
    let mut output = Vec::new();
    for &value in values {
        let clamped = Some(value).map(|n| {
            let result = if n > 0 { n } else { 0 };
            flux_rs::assert(result > 0); //~ ERROR refinement type
            result
        });
        output.push(clamped);
    }
}

pub fn false_captured_claim(values: &[i32], floor: i32) {
    for &value in values {
        let _ = Some(value).map(|n| {
            Some(n).map(|m| {
                let result = if m > floor { m } else { floor };
                flux_rs::assert(result > floor); //~ ERROR refinement type
                result
            })
        });
    }
}
