extern crate flux_core;
extern crate flux_alloc;

// Reduced from Codex agent-role nickname validation. The parent loop is visited
// repeatedly during shape inference; the closure has its own control-flow join.
pub fn validate(strings: &[String]) -> bool {
    let mut results = Vec::new();
    for value in strings {
        let value = value.trim().to_string();
        if !value.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '-' | '_')) {
            return false;
        }
        results.push(value);
    }
    true
}

pub fn clamp_values(values: &[i32]) -> Vec<Option<i32>> {
    let mut output = Vec::new();
    for &value in values {
        let clamped = Some(value).map(|n| {
            let result = if n > 0 { n } else { 0 };
            flux_rs::assert(result >= 0);
            result
        });
        output.push(clamped);
    }
    output
}

pub fn nested_captures(values: &[i32], floor: i32) {
    for &value in values {
        let _ = Some(value).map(|n| {
            Some(n).map(|m| {
                let result = if m > floor { m } else { floor };
                flux_rs::assert(result >= floor);
                result
            })
        });
    }
}
