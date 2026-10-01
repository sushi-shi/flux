//@compile-flags: -Fstd-extern-specs=on -Fcheck-overflow=strict -Fno-panic=on
use flux_core::ensures;

fn unchecked_map(xs: &[u8]) -> Option<u8> {
    Some(()).map(|_| xs[0]) //~ ERROR possible out-of-bounds access
}

fn unchecked_map_or(xs: &[u8]) -> u8 {
    Some(()).map_or(0, |_| xs[0]) //~ ERROR possible out-of-bounds access
}

fn unchecked_and_then(xs: &[u8]) -> Option<u8> {
    Some(()).and_then(|_| Some(xs[0])) //~ ERROR possible out-of-bounds access
}

fn reversed_guard(xs: &[u8]) -> Option<u8> {
    let requested = if xs.is_empty() { Some(()) } else { None };
    requested.map(|_| xs[0]) //~ ERROR possible out-of-bounds access
}

#[ensures(result.is_some())]
fn absent_is_not_present() -> Option<u8> {
    let absent: Option<()> = None;
    absent.map(|_| 0) //~ ERROR refinement type error
}

#[ensures(result == 1)]
fn absent_uses_default() -> u8 {
    let absent: Option<()> = None;
    absent.map_or(0, |_| 1) //~ ERROR refinement type error
}
