//@compile-flags: -Fstd-extern-specs=on -Fcheck-overflow=strict -Fno-panic=on
use flux_core::{ensures, refined};
#[refined(text)]
struct Text { text: String }

#[ensures(result == text.text.is_char_boundary(offset))]
fn wrong(text: &Text, offset: usize) -> bool {
    !text.text.is_char_boundary(offset) //~ ERROR refinement type
}

#[ensures(result.is_none_or(|offset| text.text.is_char_boundary(offset)))]
fn invalid(text: &Text) -> Option<usize> {
    Some(text.text.len() + 1) //~ ERROR refinement type
                             //~| ERROR refinement type
}
