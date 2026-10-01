use std::fmt::Display;

extern crate alloc;

use flux_attrs::*;

/// A closed-by-default capability for the standard ToString blanket impl.
/// Unknown Display implementations may panic or return fmt::Error.
/// Opting in is an external model obligation, not a proof about arbitrary fmt.
#[extern_spec(core::fmt)]
#[assoc(fn string_conversion_no_panic() -> bool { false })]
trait Display {}

/// Audited against pinned Rust 8925ea358a0f (alloc/src/string.rs): the
/// `to_string_str!` specialization for str calls String::from(&str), copying an
/// existing, valid allocation-sized slice. It invokes no user Display code.
/// Allocation failure can abort; this model only rules out Rust panics.
#[extern_spec(core::fmt)]
#[assoc(fn string_conversion_no_panic() -> bool { true })]
impl Display for str {}

#[extern_spec(alloc::string)]
impl<T: Display + ?Sized> ToString for T {
    #[no_panic_if(<T as Display>::string_conversion_no_panic())]
    #[spec(fn(&T) -> String)]
    fn to_string(&self) -> String;
}

#[extern_spec]
#[flux::refined_by(val: str)]
struct String;

#[extern_spec]
#[assoc(
    fn is_eq(x: String, y: String, res: bool) -> bool { res <=> (x.val == y.val) }
    fn is_ne(x: String, y: String, res: bool) -> bool { res <=> (x.val != y.val) }
)]
impl PartialEq for String {
    #[spec(fn(&String[@s], &String[@t]) -> bool[s == t])]
    fn eq(&self, other: &String) -> bool;
}

#[extern_spec]
#[assoc(fn cloned(old: Self, new: Self) -> bool { old == new } )]
impl Clone for String {
    #[spec(fn (&String[@s]) -> String[s])]
    fn clone(&self) -> Self;
}

#[trusted]
#[spec(fn(&str[@s]) -> String[s])]
pub fn mk_string(s: &str) -> String {
    s.to_string()
}
