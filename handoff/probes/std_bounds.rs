// Bounds checks through std specs. Needs `-- -Fstd-extern-specs`; without it Vec indexing is unchecked.
pub fn vec_idx(v: Vec<usize>, i: usize) -> usize { v[i] }                   // rejected
pub fn vec_ok(v: Vec<usize>) -> usize { if v.len() > 0 { v[0] } else { 0 } } // accepted
pub fn slice_idx(v: &[usize], i: usize) -> usize { v[i] }                  // rejected
pub fn pushed(n: usize) -> usize {                                           // rejected (index == len)
    let mut v = Vec::new();
    for k in 0..n { v.push(k); }
    v[n]
}
