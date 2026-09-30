// Reduced from Codex's agent-role validation. Borrow-check MIR includes fake
// borrows of Some's field even on the None path; those are not runtime accesses.

#[flux::sig(fn(Option<&str>, bool) -> u8{v: v <= 2})]
pub fn guarded_pattern(input: Option<&str>, required: bool) -> u8 {
    match input {
        Some("") => 0,
        Some(_) => 1,
        None if required => 2,
        None => 0,
    }
}

#[flux::sig(fn(Option<&str>) -> u8{v: v <= 2})]
pub fn string_pattern(input: Option<&str>) -> u8 {
    match input {
        Some("") => 0,
        Some(_) => 1,
        None => 2,
    }
}

#[flux::sig(fn(Option<Result<&str, u32>>) -> u8{v: v <= 3})]
pub fn nested_variants(input: Option<Result<&str, u32>>) -> u8 {
    match input {
        Some(Ok("")) => 0,
        Some(Ok(_)) => 1,
        Some(Err(_)) => 2,
        None => 3,
    }
}
