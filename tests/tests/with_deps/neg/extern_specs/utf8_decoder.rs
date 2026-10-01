//@compile-flags: -Fstd-extern-specs=on -Fcheck-overflow=strict -Fno-panic=on
fn incomplete_can_be_three_bytes(bytes: &[u8]) {
    if let Err(error) = std::str::from_utf8(bytes) {
        if error.error_len().is_none() {
            assert!(bytes.len() - error.valid_up_to() <= 2); //~ ERROR may panic
        }
    }
}

fn nonempty_is_not_necessarily_valid(bytes: &[u8]) {
    assert!(std::str::from_utf8(bytes).is_ok()); //~ ERROR may panic
}
