fn check_decoder(bytes: &[u8]) {
    match std::str::from_utf8(bytes) {
        Ok(text) => assert_eq!(text.len(), bytes.len()),
        Err(error) => {
            let valid = error.valid_up_to();
            assert!(valid < bytes.len());
            assert!(std::str::from_utf8(&bytes[..valid]).is_ok());
            match error.error_len() {
                Some(n) => assert!((1..=3).contains(&n) && valid + n <= bytes.len()),
                None => assert!((1..=3).contains(&(bytes.len() - valid))),
            }
        }
    }
}

#[test]
fn decoder_bounds_cover_all_two_byte_inputs_and_unicode_prefixes() {
    check_decoder(&[]);
    for bytes in [
        &b"x\xf0\x90\x80\xff"[..],
        &b"x\xe0\xa0\xff"[..],
        &b"x\xc2\xff"[..],
        &b"\xed\xa0\x80"[..],
        &b"\xf4\x90\x80\x80"[..],
    ] {
        check_decoder(bytes);
    }
    for first in 0..=255u8 {
        check_decoder(&[first]);
        for second in 0..=255u8 {
            check_decoder(&[first, second]);
        }
    }
    for scalar in 0..=0x10ffff {
        if let Some(ch) = char::from_u32(scalar) {
            let mut storage = [0; 4];
            let bytes = ch.encode_utf8(&mut storage).as_bytes();
            for end in 1..=bytes.len() {
                check_decoder(&bytes[..end]);
            }
        }
    }
}

#[test]
fn drain_bound_survives_drop_partial_iteration_and_leaking() {
    for len in 0..32 {
        for prefix in 0..=len {
            for consume in 0..=prefix {
                let mut normal = vec![0u8; len];
                let mut drain = normal.drain(..prefix);
                for _ in 0..consume {
                    let _ = drain.next();
                }
                drop(drain);
                assert_eq!(normal.len(), len - prefix);
                let mut leaked = vec![0u8; len];
                let mut drain = leaked.drain(..prefix);
                for _ in 0..consume {
                    let _ = drain.next();
                }
                std::mem::forget(drain);
                assert!(leaked.len() <= len - prefix);
            }
        }
    }
}

#[test]
fn caught_unwind_can_expose_a_broken_normal_return_invariant() {
    let mut count = 0;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        count = 10;
        panic!("state is temporarily invalid");
    }));
    assert!(result.is_err());
    assert_eq!(count, 10);
}
