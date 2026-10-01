//@compile-flags: -Fcheck-overflow=strict -Fno-panic=on
extern crate flux_alloc;
extern crate flux_core;
use std::collections::BTreeMap;

#[flux::sig(fn() -> BTreeMap<u32, Vec<u8>>[set_empty(0), map_default(0), 0, 0])]
fn empty() -> BTreeMap<u32, Vec<u8>> { BTreeMap::new() }

#[flux::sig(fn(m: &strg BTreeMap<u32, Vec<u8>>[@keys, @weights, @total, @len], key: u32, payload: Vec<u8>[@size])
    requires !set_is_in(key, keys) && len < usize::MAX
    ensures m: BTreeMap<u32, Vec<u8>>[set_union(keys, set_singleton(key)), map_store(weights, key, size), total + size, len + 1])]
fn insert_new(m: &mut BTreeMap<u32, Vec<u8>>, key: u32, payload: Vec<u8>) {
    assert!(!m.contains_key(&key));
    assert!(m.insert(key, payload).is_none());
}

#[flux::sig(fn(m: &strg BTreeMap<u32, Vec<u8>>[@keys, @weights, @total, @len], key: u32,
              bytes: &strg usize[total])
    ensures m: BTreeMap<u32, Vec<u8>>[#new_keys, #new_weights, #new_total, #new_len], bytes: usize[new_total])]
fn remove_and_charge(m: &mut BTreeMap<u32, Vec<u8>>, key: u32, bytes: &mut usize) {
    if let Some(payload) = m.remove(&key) { *bytes -= payload.len(); }
}

fn replacement_preserves_key_count_and_returns_the_old_payload() {
    let mut map = BTreeMap::new();
    assert!(map.insert(-2i32, vec![0u8, 1]).is_none());
    if let Some(old) = map.insert(-2i32, vec![0u8]) {
        assert!(old.len() == 2);
    } else {
        unreachable!();
    }
    assert!(map.len() == 1);
    assert!(map.remove(&3).is_none());
    if let Some(payload) = map.remove(&-2) {
        assert!(payload.len() == 1);
    } else {
        unreachable!();
    }
    assert!(map.len() == 0);
}
