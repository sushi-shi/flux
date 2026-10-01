fn main() {
    let samples = ["", "a", "é", "aé日", "🙂abc🙂", "abab"];
    let mut cases = 0;
    for text in samples {
        for needle in samples {
            if let Some(start) = text.find(needle) {
                assert!(text.is_char_boundary(start));
                assert!(text.is_char_boundary(start + needle.len()));
            }
            if text.ends_with(needle) {
                assert!(text.is_char_boundary(text.len() - needle.len()));
            }
            cases += 1;
        }
        for end in 0..=text.len() {
            if !text.is_char_boundary(end) { continue; }
            let expected = &text[end..];
            let mut dropped = text.to_owned();
            drop(dropped.drain(..end));
            assert_eq!(dropped, expected);
            let mut partial = text.to_owned();
            let mut drain = partial.drain(..end);
            let _ = drain.next();
            drop(drain);
            assert_eq!(partial, expected);
            let mut forgotten = text.to_owned();
            std::mem::forget(forgotten.drain(..end));
            assert_eq!(forgotten, text);
            cases += 3;
        }
    }
    println!("{cases} native search/drain checks passed");
}
