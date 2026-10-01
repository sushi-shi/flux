#[test]
fn string_append_and_clear_preserve_content_and_byte_length() {
    for left in ["", "a", "é", "🦀", "hello"] {
        for right in ["", "β", "a\0b", "tail"] {
            let mut text = String::new();
            text.push_str(left);
            text.push_str(right);
            assert_eq!(text.as_str(), format!("{left}{right}"));
            assert_eq!(text.len(), left.len() + right.len());
            assert_eq!(text.is_empty(), left.is_empty() && right.is_empty());
            text.clear();
            assert_eq!(text.as_str(), "");
            assert_eq!(text.len(), 0);
        }
    }
}
