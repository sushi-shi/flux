//! Native falsification checks for the UTF-8 extern model, not a soundness proof.

#[test]
fn width_model_covers_every_unicode_scalar_value() {
    for value in 0..=0x10ffff {
        let Some(ch) = char::from_u32(value) else { continue };
        let expected = if value < 0x80 { 1 } else if value < 0x800 { 2 }
            else if value < 0x10000 { 3 } else { 4 };
        assert_eq!(ch.len_utf8(), expected);
    }
}

#[test]
fn cursor_and_slice_models_match_utf8_strings() {
    for text in ["", "ascii", "\0é中😀", "a\u{301}", "👩\u{200d}💻", "\u{10ffff}"] {
        assert!(text.len() <= isize::MAX as usize);
        assert!(text.is_char_boundary(0));
        assert!(text.is_char_boundary(text.len()));
        let mut iter = text.char_indices();
        let mut remaining = text.chars().count();
        assert!(remaining <= text.len());
        loop {
            let offset = iter.offset();
            assert!(text.is_char_boundary(offset));
            match iter.next() {
                Some((index, ch)) => {
                    assert!(remaining > 0);
                    remaining -= 1;
                    assert_eq!(index, offset);
                    assert!(text.is_char_boundary(index + ch.len_utf8()));
                    assert!(iter.offset() > offset);
                }
                None => {
                    assert_eq!(remaining, 0);
                    assert_eq!(iter.offset(), offset);
                    assert_eq!(iter.next(), None);
                    assert_eq!(iter.offset(), offset);
                    break;
                }
            }
        }
        for at in 0..=text.len() {
            if text.is_char_boundary(at) {
                let (left, right) = (&text[..at], &text[at..]);
                assert_eq!(left.len(), at);
                assert_eq!(right.len(), text.len() - at);
                assert!(text.starts_with(left));
                assert!(text.ends_with(right));
                for end in at..=text.len() {
                    if text.is_char_boundary(end) {
                        let part = &text[at..end];
                        assert_eq!(part.len(), end - at);
                        assert!(text.contains(part));
                    }
                }
            }
        }
    }
}
