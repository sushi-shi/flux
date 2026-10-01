# Collection models exercised by the Codex relay

The model was audited against pinned Rust `8925ea358a0f` (nightly 2026-08-21).
External specifications are trusted assumptions, not proofs of the standard library.

`BTreeMap` tracks key membership, a per-key nonnegative weight, total weight, and length.
The proof-only `ValueWeight` trait specializes Vec values to their length; unknown types
retain an uninterpreted measure. Replacement subtracts the old weight and adds the new one.
Removal subtracts a weight only for a present key and returns that value. Empty maps have
zero total. These properties do not describe ciphertext contents or iteration order.

Key encoding and panic guarantees require an audited primitive integer `Ord` implementation.
Borrowed queries additionally require identity `Borrow<T> for T`. Unknown comparators and
borrow implementations default to disabled. `contains_key` only searches; `remove` returns
the value and drops an integer key. Allocation/deallocation and allocator cloning require
a separate default-false capability, enabled only for `Global`. Its backend `GlobalAlloc`
contract forbids unwinding; allocation failure may abort. Integer-key B-tree nodes occupy
more than one byte per entry including node metadata, so a live map cannot reach an
overflowing usize entry count before Global allocation fails. Node layouts are statically
sized. Sources: `alloc/src/collections/btree/{map,node}.rs`, `alloc/src/alloc.rs`, and
`core/src/alloc/global.rs`. This does not claim termination or allocation success.

Vec's invariant bounds allocated bytes by `isize::MAX`, not its element count: zero-sized
elements permit `usize::MAX` length. Panic-free push requires room for one more element
and, for nonzero-sized elements, for `RawVec::grow_amortized`'s doubled capacity and minimum
initial allocation. The guard follows `alloc/src/raw_vec/mod.rs`; merely bounding the next
length would be unsound. Box's `new_uninit` uses a valid static layout and Global allocation.
The existing boxed-array-to-Vec helper transfers ownership without allocating; its existing
initialization assumption remains trusted.

Positive and negative verifier cases cover replacement, removal, accounting, unknown keys,
allocators and weights, zero-sized elements, and capacity doubling. Native tests compare
20,000 map operations with independent slots and exercise the zero-sized length boundary.
Codex's relay is checked with strict overflow and panic checking. Other targets, feature
configurations, callers, dependency bodies, payload content, and full ordering remain outside
that result.

The readable `#[refined]` macro infers structural annotations; it adds no axioms or trust.
Its negative cases check invalid constructors, false frame conditions, and same-named user
container types. Ordinary builds erase it and preserve the Rust representation.

An abandoned per-instance predicate-weight constructor prototype triggered a checker panic
in fixpoint encoding (`unexpected $k4(a0.0, a1)`). The type-defined measure avoids that path;
it does not fix or establish support for output-only higher-order predicate inference.
