//@compile-flags: -Fcheck-overflow=strict -Fno-panic=on
#![feature(allocator_api)]
extern crate flux_alloc;
extern crate flux_core;
use std::collections::BTreeMap;

// Replacement must charge the new payload, not keep the old size.
fn replacement() {
    let mut m = BTreeMap::new();
    m.insert(1u32, vec![0u8]);
    m.insert(1u32, vec![0u8, 1]);
    if let Some(payload) = m.remove(&1) {
        assert!(payload.len() == 1); //~ ERROR may panic
    }
}

// Removing a missing key cannot erase an unrelated payload.
fn missing_key() {
    let mut m = BTreeMap::new();
    m.insert(1u32, vec![0u8]);
    m.remove(&2);
    assert!(m.len() == 0); //~ ERROR may panic
}

fn unknown_comparator<K: Ord>(m: &BTreeMap<K, Vec<u8>>, key: &K) {
    m.contains_key(key); //~ ERROR refinement type
    //~^ ERROR may panic
}

fn unknown_allocator<A: std::alloc::Allocator + Clone>(m: &mut BTreeMap<u32, Vec<u8>, A>) {
    m.remove(&0); //~ ERROR may panic
}

#[flux::sig(fn(&BTreeMap<u32, T>{m: m.total == 0}))]
fn require_zero_total<T>(_: &BTreeMap<u32, T>) {}

fn unknown_weight<T>(value: T) {
    let mut m = BTreeMap::new();
    m.insert(0u32, value);
    require_zero_total(&m); //~ ERROR refinement type
}
