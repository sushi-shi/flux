//@compile-flags: -Fstd-extern-specs=on -Fcheck-overflow=strict -Fno-panic=on
fn bounds(bytes: &[u8]) {
    match std::str::from_utf8(bytes) {
        Ok(text) => assert!(text.len() == bytes.len()),
        Err(error) => {
            let valid = error.valid_up_to();
            assert!(valid < bytes.len());
            match error.error_len() {
                Some(n) => { assert!(1 <= n && n <= 3); assert!(valid + n <= bytes.len()); }
                None => { assert!(1 <= bytes.len() - valid && bytes.len() - valid <= 3); }
            }
        }
    }
}

fn empty() { assert!(std::str::from_utf8(&[]).is_ok()); }
