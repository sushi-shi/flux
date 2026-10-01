//@compile-flags: -Fno-panic=on
extern crate flux_alloc;
extern crate flux_core;

fn text_conversion(text: &str) -> String { text.to_string() }
fn qualified_conversion(text: &str) -> String { <str as ToString>::to_string(text) }
