//@compile-flags: -Fcheck-overflow=strict -Fno-panic=on
extern crate flux_alloc;
extern crate flux_core;

#[flux::sig(fn(v: &Vec<u8>[@n]) -> usize[n + 1048576])]
fn payload_plus_buffer(v: &Vec<u8>) -> usize { v.len() + 1048576 }

#[flux::sig(fn(v: &mut Vec<()>[@n]) requires n < usize::MAX
    ensures v: Vec<()>[n + 1])]
fn push_zst(v: &mut Vec<()>) { v.push(()); }

#[flux::sig(fn(v: &mut Vec<u8>[@n]) requires n <= 1024
    ensures v: Vec<u8>[n + 1])]
fn small_push(v: &mut Vec<u8>) { v.push(0); }
