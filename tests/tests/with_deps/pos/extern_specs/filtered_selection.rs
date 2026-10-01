//@compile-flags: -Fstd-extern-specs=on -Fcheck-overflow=strict -Fno-panic=on
use flux_core::{ensures, may_panic};
#[may_panic]
#[ensures(result.is_none_or(|(_, index)| index < xs.len()))]
fn triple(xs: &[u8]) -> Option<(usize, usize)> {
    xs.iter().enumerate()
      .filter_map(|(i, _)| Some((0usize, 1usize, i)))
      .min_by(|(a, b, c), (d, e, f)| a.cmp(d).then_with(|| e.cmp(b)).then_with(|| c.cmp(f)))
      .map(|(a, _, c)| (a, c))
}
#[may_panic]
#[ensures(result.is_none_or(|(_, index)| index < xs.len()))]
fn search(xs: &[&str], pending: &str) -> Option<(usize, usize)> {
    xs.iter().enumerate()
      .filter_map(|(i, spec)| pending.find(*spec).map(|p| (p, spec.len(), i)))
      .min_by(|(a, b, c), (d, e, f)| a.cmp(d).then_with(|| e.cmp(b)).then_with(|| c.cmp(f)))
      .map(|(a, _, c)| (a, c))
}

#[may_panic]
#[ensures(result.is_none_or(|flag| flag))]
fn boolean(n: usize) -> Option<bool> {
    (0..n).filter_map(|i| Some(i < n)).min_by(|_, _| core::cmp::Ordering::Equal)
}

#[ensures(result.is_none_or(|offset| offset <= text.len() && text.is_char_boundary(offset)))]
fn search_boundary(text: &str, needle: &str) -> Option<usize> {
    text.find(needle)
}

fn compare(a: usize, b: usize, c: usize) -> core::cmp::Ordering {
    a.cmp(&b).then_with(|| b.cmp(&c))
}
