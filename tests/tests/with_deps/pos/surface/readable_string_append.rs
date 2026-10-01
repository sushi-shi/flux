//@compile-flags: -Fstd-extern-specs=on -Fcheck-overflow=strict -Fno-panic=on
use flux_core::{ensures, may_panic, refined};

#[refined(text)]
struct Output<T> { text: String, items: Vec<T> }

#[may_panic]
#[ensures(out.text == concat(old(out.text), text))]
fn append<T>(out: &mut Output<T>, text: &str) {
    if !text.is_empty() { out.text.push_str(text); }
}

// Composition belongs here as a verifier regression; the application contract
// above describes the operation's actual output.
#[may_panic]
#[ensures(out.text == concat(concat(old(out.text), first), second))]
fn compose<T>(out: &mut Output<T>, first: &str, second: &str) {
    append(out, first);
    append(out, second);
}
