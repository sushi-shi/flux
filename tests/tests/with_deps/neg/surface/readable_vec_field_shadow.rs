//@compile-flags: -Fstd-extern-specs=on
use flux_core::{ensures, refined};
#[refined]
struct Impostor { len: usize }
impl Impostor { fn len(&self) -> usize { 100 } }
#[refined]
struct Outer { field: Impostor }

// A custom field with a len member must not inherit the standard Vec model.
#[ensures(result == value.field.len())] //~ ERROR trait bound
fn custom_length(value: &Outer) -> usize { 0 }
