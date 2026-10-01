//@compile-flags: -Fstd-extern-specs=on
use flux_rs::attrs::*;
use flux_rs::assert;
extern crate flux_core;

// An item model can be verified without claiming an exact remaining size.
struct Seven { pending: bool }
#[assoc(fn valid_item(self: Self, item: int) -> bool { item == 7 })]
impl Iterator for Seven {
    type Item = usize;
    #[spec(fn(self: &mut Self) -> Option<usize[7]> ensures self: Self)]
    fn next(&mut self) -> Option<usize> {
        if self.pending { self.pending = false; Some(7) } else { None }
    }
}
fn seven_items() {
    Seven { pending: true }.for_each(|item| assert(item == 7));
}

#[refined_by(remaining: int)]
struct Countdown {
    #[field(usize[remaining])]
    remaining: usize,
}
#[assoc(fn valid_item(self: Self, item: int) -> bool { 0 < item && item <= self.remaining })]
impl Iterator for Countdown {
    type Item = usize;
    #[spec(fn(self: &mut Self[@before]) -> Option<usize{item: 0 < item && item <= before.remaining}>
        ensures self: Self[if before.remaining > 0 { before.remaining - 1 } else { 0 }])]
    fn next(&mut self) -> Option<usize> {
        if self.remaining > 0 {
            let item = self.remaining;
            self.remaining -= 1;
            Some(item)
        } else { None }
    }
}
fn countdown_items(limit: usize) {
    Countdown { remaining: limit }.for_each(|item| assert(item > 0 && item <= limit));
}
