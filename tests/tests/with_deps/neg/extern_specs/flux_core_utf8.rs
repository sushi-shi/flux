//@compile-flags: -Fcheck-overflow=strict -Fno-panic=on
extern crate flux_core;
use flux_rs::assert;

pub fn wrong_width() {
    assert('😀'.len_utf8() == 1); //~ ERROR refinement type
}

pub fn byte_offset_is_not_necessarily_a_boundary(s: &str, at: usize) {
    if at <= s.len() {
        let _ = &s[..at]; //~ ERROR refinement type
    }
}

pub fn wrong_slice_length(s: &str, at: usize) {
    if at > 0 && s.is_char_boundary(at) {
        assert(s[..at].len() == 0); //~ ERROR refinement type
    }
}

pub fn exhausted_iteration_does_not_prove_false() {
    let mut iter = "".char_indices();
    let _ = iter.next();
    let _ = iter.next();
    assert(false); //~ ERROR refinement type
}

pub fn inside_character(s: &str) {
    for (index, ch) in s.char_indices() {
        if ch.len_utf8() > 1 {
            let _ = &s[..index + 1]; //~ ERROR refinement type
        }
    }
}
