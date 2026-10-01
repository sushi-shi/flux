//! Bounds from the standard UTF-8 decoder; byte contents are not modeled.

use flux_attrs::*;

#[extern_spec(core::str)]
#[refined_by(valid_up_to: int, error_len: int)]
#[invariant(0 <= valid_up_to && valid_up_to <= isize::MAX)]
#[invariant(0 <= error_len && error_len <= 3)]
struct Utf8Error;

#[extern_spec(core::str)]
#[no_panic]
#[spec(fn(bytes: &[u8][@n]) -> Result<&str{s: super::byte_len(s) == n},
    Utf8Error{e: e.valid_up_to < n &&
        (if e.error_len == 0 { n - e.valid_up_to <= 3 }
         else { e.valid_up_to + e.error_len <= n })}>{result: n > 0 || result.is_ok})]
fn from_utf8(bytes: &[u8]) -> Result<&str, Utf8Error>;

#[extern_spec(core::str)]
impl Utf8Error {
    #[no_panic]
    #[spec(fn(&Self[@e]) -> usize[e.valid_up_to])]
    const fn valid_up_to(&self) -> usize;

    #[no_panic]
    #[spec(fn(&Self[@e]) -> Option<usize[e.error_len]>[e.error_len > 0])]
    const fn error_len(&self) -> Option<usize>;
}
