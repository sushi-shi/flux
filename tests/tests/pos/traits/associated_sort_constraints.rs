//! A mutation can introduce type-argument holes without changing an associated sort.
pub struct Chunk<T> {
    pub items: Vec<T>,
}

pub trait Parser {
    type Item;
    fn finish(&mut self) -> Chunk<Self::Item>;
}

pub fn finish<P: Parser>(parser: &mut P, empty: bool) -> Result<Chunk<P::Item>, ()> {
    let mut out = if empty { Chunk { items: Vec::new() } } else { parser.finish() };
    let mut tail = parser.finish();
    out.items.append(&mut tail.items);
    Ok(out)
}

