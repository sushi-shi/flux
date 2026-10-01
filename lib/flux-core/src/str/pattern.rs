//! Literal string patterns only; arbitrary predicate patterns stay disabled.
use flux_attrs::*;

#[extern_spec(core::str::pattern)]
#[assoc(
    fn literal_string() -> bool { false }
)]
trait Pattern {}

#[extern_spec(core::str::pattern)]
#[assoc(
    fn literal_string() -> bool { true }
)]
impl<'b> Pattern for &'b str {}
