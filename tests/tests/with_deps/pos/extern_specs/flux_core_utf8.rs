//@compile-flags: -Fcheck-overflow=strict -Fno-panic=on
extern crate flux_core;
use flux_rs::assert;

pub fn widths() {
    assert('a'.len_utf8() == 1);
    assert('é'.len_utf8() == 2);
    assert('中'.len_utf8() == 3);
    assert('😀'.len_utf8() == 4);
}

pub fn endpoints(s: &str) {
    let len = s.len();
    assert(s.is_char_boundary(0));
    assert(s.is_char_boundary(len));
    assert(s[..0].is_empty());
    assert(s[len..].is_empty());
    assert(s[..].len() == len);
}

pub fn guarded_split(s: &str, at: usize) {
    if s.is_char_boundary(at) {
        let left = &s[..at];
        let right = &s[at..];
        assert(left.len() == at);
        assert(left.len() + right.len() == s.len());
    }
}

pub fn character_endpoints(s: &str) {
    let len = s.len();
    for (index, ch) in s.char_indices() {
        let end = index + ch.len_utf8();
        assert(end <= len);
        assert(s.is_char_boundary(index));
        assert(s.is_char_boundary(end));
        let _ = &s[index..end];
    }
}
