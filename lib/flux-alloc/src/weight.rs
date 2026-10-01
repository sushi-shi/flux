//! Proof-only value measures used by weighted container models.
//! Unknown value types retain an uninterpreted measure; Vec contributes its length.

use flux_attrs::*;

defs! { fn opaque_weight<T>(value: T) -> int; }

#[assoc(fn weight(value: Self) -> int { opaque_weight(value) })]
pub trait ValueWeight {}

impl<T> ValueWeight for T {}

#[assoc(fn weight(value: Self) -> int { value.len })]
impl<T> ValueWeight for Vec<T> {}
