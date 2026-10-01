//@compile-flags: -Fstd-extern-specs=on -Fcheck-overflow=strict -Fno-panic=on
use flux_core::{ensures, refined, requires};
#[refined(text)]
struct Text { text: String }

#[ensures(result == text.text.is_char_boundary(offset))]
fn boundary(text: &Text, offset: usize) -> bool {
    text.text.is_char_boundary(offset)
}

#[ensures(result.is_none_or(|offset| text.text.is_char_boundary(offset)))]
fn find(text: &Text, needle: &str) -> Option<usize> {
    text.text.find(needle)
}

#[requires(text.text.is_char_boundary(offset))]
#[ensures(result == true)]
fn required(text: &Text, offset: usize) -> bool {
    text.text.is_char_boundary(offset)
}

#[ensures(result.text.is_char_boundary(0))]
fn empty() -> Text { Text { text: String::new() } }

// Matches the arithmetic used before draining a closing delimiter. Each
// byte offset/length is at most isize::MAX, so their sum fits usize.
fn match_end(text: &str, needle: &str) -> Option<usize> {
    text.find(needle).map(|offset| offset + needle.len())
}
