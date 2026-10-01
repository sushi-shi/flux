//@compile-flags: -Fstd-extern-specs=on -Fcheck-overflow=strict -Fno-panic=on
use flux_core::ensures;

#[ensures(result.is_none_or(|index| index < limit))]
fn bad_index(limit: usize) -> Option<usize> {
    Some(limit) //~ ERROR refinement type
}

#[ensures(result.is_none_or(|(position, index)| position <= limit && index < limit))]
fn bad_tuple(limit: usize) -> Option<(usize, usize)> {
    Some((0, limit)) //~ ERROR refinement type
}

#[ensures(result.is_none_or(|_| false))]
fn cannot_return_some() -> Option<usize> {
    Some(0) //~ ERROR refinement type
}

#[ensures(result.is_some())]
fn cannot_claim_present() -> Option<usize> {
    None //~ ERROR refinement type
}

#[ensures(result.is_none_or(|limit| limit == 0))]
fn shadow_must_be_checked(limit: usize) -> Option<usize> {
    Some(1) //~ ERROR refinement type
}
