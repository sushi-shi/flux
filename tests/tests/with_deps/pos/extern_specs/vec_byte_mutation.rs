//@compile-flags: -Fstd-extern-specs=on -Fcheck-overflow=strict -Fno-panic=on
#[flux::sig(fn(v: &strg Vec<u8>[@n], data: &[u8][@m]) requires n + m <= 1024
    ensures v: Vec<u8>[n + m])]
fn extend(v: &mut Vec<u8>, data: &[u8]) { v.extend_from_slice(data); }

#[flux::sig(fn(v: &strg Vec<u8>[@n], limit: usize)
    ensures v: Vec<u8>{m: m <= n && m <= limit})]
fn truncate(v: &mut Vec<u8>, limit: usize) { v.truncate(limit); }

#[flux::sig(fn(v: &strg Vec<u8>[@n]) ensures v: Vec<u8>[0])]
fn clear(v: &mut Vec<u8>) { v.clear(); }

#[flux::sig(fn(v: &strg Vec<u8>[@n], prefix: usize) requires prefix <= n
    ensures v: Vec<u8>{m: m <= n - prefix})]
fn drain(v: &mut Vec<u8>, prefix: usize) { v.drain(..prefix); }

#[flux::sig(fn(v: &strg Vec<u8>[@n], prefix: usize) requires prefix <= n
    ensures v: Vec<u8>{m: m <= n - prefix})]
fn leaked_drain(v: &mut Vec<u8>, prefix: usize) { std::mem::forget(v.drain(..prefix)); }
