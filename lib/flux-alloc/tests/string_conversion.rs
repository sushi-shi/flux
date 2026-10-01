use std::fmt;

#[test]
fn text_conversion_preserves_utf8_and_bytes() {
    for text in ["", "Noise relay sequence number exhausted", "é🦀\0\n", "東京"] {
        assert_eq!(text.to_string().as_bytes(), text.as_bytes());
    }
}

#[test]
fn arbitrary_display_is_not_panic_free() {
    struct Fails;
    impl fmt::Display for Fails {
        fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
            Err(fmt::Error)
        }
    }
    assert!(std::panic::catch_unwind(|| Fails.to_string()).is_err());
}
