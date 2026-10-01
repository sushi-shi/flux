//@compile-flags: -Fstd-extern-specs=on
use flux_rs::attrs::*;
use flux_rs::assert;
extern crate flux_core;

#[refined_by(limit: int)]
struct One {
    #[field(usize[limit])]
    limit: usize,
    pending: bool,
}

#[assoc(fn valid_item(self: Self, item: (int, bool)) -> bool {
    item.0 < self.limit && item.1
})]
impl Iterator for One {
    type Item = (usize, bool);

    #[spec(fn(self: &mut Self[@s]) -> Option<(usize{v: v < s.limit}, bool[true])>
        ensures self: Self[s.limit])]
    fn next(&mut self) -> Option<(usize, bool)> {
        if self.pending && self.limit > 0 {
            self.pending = false;
            Some((0, true))
        } else {
            None
        }
    }
}

fn callback(limit: usize) {
    One { limit, pending: true }.for_each(|(index, present)| {
        assert(index < limit && present);
    });
}
