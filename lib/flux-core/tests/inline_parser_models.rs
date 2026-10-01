#[test]
fn inclusive_bounds_hold_with_reverse_and_mixed_consumption() {
    for lower in -8..=8i32 {
        for upper in -8..=8i32 {
            let mut range = lower..=upper;
            let mut actual = Vec::new();
            while let Some(back) = range.next_back() {
                actual.push(back);
                if let Some(front) = range.next() {
                    actual.push(front);
                }
            }
            actual.sort();
            let expected: Vec<_> = (lower..=upper).collect();
            assert_eq!(actual, expected);
            assert!(actual.iter().all(|v| lower <= *v && *v <= upper));
        }
    }
    assert_eq!(
        (usize::MAX - 1..=usize::MAX).rev().collect::<Vec<_>>(),
        vec![usize::MAX, usize::MAX - 1]
    );
    assert_eq!((i32::MIN..=i32::MIN + 1).rev().collect::<Vec<_>>(), vec![i32::MIN + 1, i32::MIN]);
}

#[test]
fn literal_suffix_matching_covers_unicode_empty_and_overlapping_patterns() {
    for text in ["", "a", "aaa", "é", "αβ", "a🦀é"] {
        for needle in ["", "a", "aa", "é", "β", "🦀é", "αβ", "longer"] {
            if text.ends_with(needle) {
                assert!(needle.len() <= text.len());
                assert!(text.is_char_boundary(text.len() - needle.len()));
            }
        }
    }
}
