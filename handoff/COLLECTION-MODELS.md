# Collection models exercised by the Codex relay and UTF-8 parser

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

## UTF-8 stream state

`core::str::from_utf8` preserves byte length on success. Its error has a valid prefix
strictly shorter than the input; a known invalid sequence occupies one to three bytes
within the input, and an incomplete suffix occupies one to three bytes. Empty input
succeeds. These facts follow `core/src/str/{converts,error,validations}.rs` at the same
pinned Rust revision. The model does not describe byte contents. Native checks cover
all one- and two-byte inputs, every Unicode scalar's UTF-8 prefixes, and selected longer
invalid sequences; these execution checks supplement the audit rather than prove it.

Byte Vec `clear`, `truncate`, and `extend_from_slice` have length postconditions.
The last operation requires the byte-copy capability, and clear/truncate gain their
non-panicking destructor guarantee only for bytes. Unknown Clone/Drop implementations
do not gain it. Extending without panic additionally needs Global allocation and the
conservative doubled-growth bound `max(8, 2 * new_len) <= isize::MAX`. The actual parser
permits panics instead of imposing an artificial small-chunk precondition.

Prefix `drain(..end)` requires `end <= old_len` and leaves length at most `old_len - end`.
This is deliberately an upper bound: forgetting the iterator can leave length zero;
normal Drop restores the tail. `Drain::keep_rest` is rejected because it can restore
removed elements and requires a model connecting the iterator with its owner. Other
range implementations default to disabled. Native checks cover normal Drop, partial
iteration, and forgetting. Indexing retains the sealed SliceIndex bounds obligations.

The readable field-length frontend uses a Rust type witness restricted to the canonical
Vec type. A typed logical helper alone was insufficient because single-field refinement
records can coerce through their scalar component. The witness prevents fake containers
with a custom `len()` method from receiving these trusted semantics.

The parser's normal-return contracts bound the pending length, preserve it on error,
and empty it after successful finishing. Two checked lemmas exercise construction and
repeated errors. They do not prove buffer content, callback history, or arbitrary
dependency bodies. The native parser test independently checks exact contents and
rollback for all 1,024 chunk partitions of `Aé中🦀Z`.

A caught callback panic can expose six buffered bytes, violating the normal-return
three-byte invariant. A minimized Flux example initially accepted a false postcondition
after `catch_unwind`. The checker now inserts an unsatisfiable precondition at that
standard boundary, even without model crates and when reified to a function pointer.
This is fail-closed handling of an unsupported boundary, not an unwind proof. Other
recovery/concurrency mechanisms and general destructor effects remain outside this result.

## Concrete inline parser

`RangeInclusive` carries an immutable envelope of the original lower and upper endpoints.
Forward and backward yielded items stay in that envelope, including the upper endpoint.
The enabled `Step` capability is restricted to audited usize/i32 implementations; unknown
implementations remain disabled. The model deliberately grants no exact remaining-size or
exhaustion facts: endpoints returned by Rust's accessors can change during iteration, and
must not be identified with these ghost bounds. `Rev::next` delegates modeled backward
item/transition facts. Generic backwards iteration requires an explicit capability; it
does not silently inherit a decreasing count. Audit sources are pinned Rust
`core/src/iter/{range,adapters/rev,traits/double_ended}.rs` and `core/src/ops/range.rs`.
Native tests mix front/back consumption and include usize::MAX and i32::MIN boundaries.

For `str::ends_with`, only the audited `Pattern for &str` receives a non-panicking
capability. Predicate patterns remain disabled. No logical matching/content relation is
assumed: the generic reference-pattern sort currently loses the string value. An attempted
associated-refinement text projection exposed that mismatch and was removed. This blocks
propagating suffix matches to buffered-text slice boundaries, rather than inventing the fact.

String's existing text model now supports empty construction, byte length, emptiness,
clearing, exact dereference, and exact concatenation on normal return from push_str.
Its byte length is nonnegative and allocation-bounded. Appending does not gain a blanket
panic-free claim; capacity growth can panic. These models were audited against
`alloc/src/string.rs` and checked natively with empty, Unicode, and embedded-NUL strings.

The inline parser's finish contract clears pending text and active-tag state, returns one
extraction iff a tag was active, and returns the entry pending text iff no tag was active.
Extraction payload contents are not yet modeled. Repeated finishing returns an empty chunk.
The suffix helper is checked for numerical bounds and a delimiter UTF-8 boundary; maximality
and the corresponding buffered-text boundary remain open. Native parser tests independently
check content and repeated finishing across all character-boundary partitions of five
complete/incomplete-tag inputs. These are distinct from proofs of push_str or the whole crate.
