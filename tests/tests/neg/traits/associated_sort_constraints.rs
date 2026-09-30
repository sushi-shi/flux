pub trait Parser {
    type Item;
}

// Erasing constraints from sort identities must not erase value refinements.
#[flux::sig(fn(&[P::Item][@n]) -> usize[n])]
pub fn wrong_length<P: Parser>(_: &[P::Item]) -> usize {
    0 //~ ERROR refinement type
}

#[flux::sig(fn(i32{v: v > 0}))]
pub fn needs_positive(_: i32) {}

pub fn invalid_call<P: Parser>(_: Option<P::Item>) {
    needs_positive(0); //~ ERROR refinement type
}
