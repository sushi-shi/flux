use flux_core::{ensures, refined};
#[refined(value)]
struct Custom { value: String }
impl Custom { fn is_char_boundary(&self, _: usize) -> bool { true } }
impl std::ops::Deref for Custom {
    type Target = String;
    fn deref(&self) -> &String { &self.value }
}
#[refined(text)]
struct Text { text: Custom }
#[ensures(text.text.is_char_boundary(0))] //~ ERROR trait bound
fn custom_is_not_string(text: &Text) {}
