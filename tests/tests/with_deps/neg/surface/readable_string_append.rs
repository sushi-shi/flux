//@compile-flags: -Fstd-extern-specs=on -Fcheck-overflow=strict -Fno-panic=on
use flux_core::{ensures, may_panic, refined};

#[refined(text)]
struct Output { text: String }

#[may_panic]
#[ensures(out.text == concat(old(out.text), text))]
fn dropped(out: &mut Output, text: &str) { //~ ERROR refinement type
}

#[may_panic]
#[ensures(out.text == concat(text, old(out.text)))]
fn wrong_order(out: &mut Output, text: &str) { //~ ERROR refinement type
    out.text.push_str(text);
}
