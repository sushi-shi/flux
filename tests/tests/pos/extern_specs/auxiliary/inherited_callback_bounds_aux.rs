pub struct Callbacks<F, G> {
    pub first: F,
    pub second: G,
}

impl<B, C, F: FnMut(usize) -> B, G: FnMut() -> C> Callbacks<F, G> {
    pub fn call_both(&mut self) -> (B, C) {
        ((self.first)(0), (self.second)())
    }
}
