#[flux::sig(fn(Option<&str>, bool) -> u8[0])]
pub fn wrong_guarded_result(input: Option<&str>, required: bool) -> u8 {
    match input {
        Some("") => 0,
        Some(_) => 1,          //~ ERROR refinement type
        None if required => 2, //~ ERROR refinement type
        None => 0,
    }
}

pub fn real_indexing_still_needs_a_bound(input: Option<&str>, values: &[u8]) -> u8 {
    match input {
        Some("") => values[0], //~ ERROR assertion might fail
        Some(_) => 1,
        None => 0,
    }
}
