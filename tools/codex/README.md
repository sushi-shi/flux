# Codex verification corpus

Run real Codex code, reduce failures, add a general model, lemma, or checker fix,
and rerun the affected code. Expand this loop until the whole tool has meaningful
contracts and accounted-for proof obligations. The package inventory keeps work
that has not yet been attempted visible. Function journals add observations for
each compiler invocation; they do not establish that intended behavior has been
fully specified.

Use the pinned environment from the Flux root:

```sh
nix develop ./handoff
cargo x build-sysroot
cargo build -p flux-bin --bin cargo-flux
python3 tools/codex/corpus.py prepare \
  --source /path/to/codex \
  --revision b4c864dd6497ae764e6a826300b34f7ca77ba965 \
  --destination /tmp/codex-flux
python3 tools/codex/corpus.py inventory \
  --source /tmp/codex-flux --output /tmp/codex-inventory.json
python3 tools/codex/corpus.py check \
  --source /tmp/codex-flux --flux "$PWD" \
  --package codex-utils-fuzzy-match \
  --package codex-utils-string \
  --package codex-utils-stream-parser \
  --output /tmp/codex-flux-runs/initial
```

`prepare` creates an isolated local clone with shared Git objects; keep its source
repository available. Checks require this prepared clone and restore the package
manifest after enabling Flux temporarily. Commit new source files in the corpus
clone before checking; tracked edits are captured as a patch with the results.
The original Codex checkout is read only throughout preparation and checking.

Each run needs a new output directory. It contains the whole workspace inventory,
raw diagnostics, available timing dumps, source patches, tool identities, model
hashes, and a JSON report. Build artifacts are reused in the output directory's
parent. A unique logging flag forces a new compiler invocation for each checked
package, so a warm Cargo build is not mistaken for a fresh verification.

Add `--offline` after dependencies have been fetched. Both metadata resolution
and compilation honor it and `--locked`. To expand the baseline, replace package
selection with `--all-libraries`, or use `--all-packages --targets all` to also
check binaries, examples, benches, and test targets. Cargo may skip targets whose
required features are disabled; these stay in the inventory. `--timeout 300`
bounds each package, including dependency compilation. A timeout or interruption
stops the owned Cargo process group (including compiler and solver children),
restores the temporary manifest, and retains available evidence. To reduce a failure, use
`--only-check 'def:function_name'`; the report records this restriction.

The initial configuration enables library models, strict arithmetic checking,
and panic obligations on the host platform with default features. Library targets
are the default. `--targets all` checks available additional targets, including
test bodies, without executing runtime tests. Other feature combinations and
platforms remain explicit inventory entries, not verified coverage. General
iterator model assumptions still need auditing, so an accepted check is only an
observation under the loaded models, not a correctness certification.

A failed obligation is not a confirmed Codex bug. Reproduce a claimed bug through
the public API. Preserve positive examples and known false claims when adding a
model or lemma; runtime checks can falsify models but cannot prove them sound.

The first actual run at the pinned revision exposed:

- Missing Cargo target and lock-policy forwarding, now covered by CLI tests.
- Panic and arithmetic obligations in `fuzzy_match`.
- Missing string position facts in `codex-utils-string`.
- A Flux crash while resolving the size of a generic type in the stream parser.

The next change is chosen from these failures, not from a fixed feature sequence.

Run the reporting tests with:

```sh
python3 -m unittest discover -s tools/codex -v
cargo test -p flux-bin --lib
```

## Iterator model prerequisite

Size-based iterator contracts now require `Iterator::has_size_model()`. The
default is false; reviewed finite models opt in and adapters propagate the
prerequisite. Generic functions that count an iterator must state this requirement
explicitly. This prevents an unrefined iterator from inheriting both an unchanged
unit index and a decreasing remaining size, which previously made false claims
provable. The regression uses filtered slices with arbitrary, unrelated lengths.
Infinite iterators cannot satisfy the finite-size requirement either.

`take(n)` still permits bounded iteration over an unknown or infinite iterator:
its model tracks the remaining bound without claiming an exact inner size or
inner transition. Reversed integer ranges have size zero, not a negative size.

This is a conservative capability boundary, not a complete iterator model.
Missing capabilities produce obligations; they are not proof success. A separate
escaping-bound-variable crash with `filter(...).enumerate()` remains unresolved.
The false count proof is retained as a negative regression independently of that
crash. Future models must include counterexamples as well as accepted programs.

## Function evidence

The [review checkpoint](OPENAI-REVIEW.md) connects the specification and draft PR
stack to a retained 152-package baseline, diagnostics, and mutation/performance
evidence. It explicitly distinguishes observations from completed specifications.

`-Fcoverage=on` (or `--Fcoverage=true` through Cargo) writes an append-only JSONL
journal under the log directory. It inventories active local functions, methods,
and closures before checking bodies, including source locations and explicit
contract text. A start, inventory-complete marker, per-item outcomes, and a final
marker distinguish complete observations from interrupted compilation. Invocation
IDs prevent overwriting repeated crate checks. The corpus runner preserves these
journals and writes `function-map.json` beside each package log.

Outcomes distinguish accepted checks under loaded models, check errors, caught
checker crashes, interrupted or unattempted bodies, trusted functions, ignored
items, unselected items, and declarations without bodies. Closures remain
obligations of the enclosing body; they do not get independent proof credit.
Lean results are conservatively left unattributed by this reporter. A complete
journal can describe a failed crate; completeness refers only to observations.

The map covers the active compiler configuration, not cfg-disabled source.
Dependency proof closure, external models, contract adequacy, domain effects,
other platforms, and other targets still need review. Even an explicit contract
is not automatically a meaningful behavioral specification. An accepted check
without an explicit contract is recorded as such, never as fully specified code.
Multiple invocations are kept separate rather than added into a coverage percent.

Exercise real compiler reporting, including an intentionally false contract:

```sh
python3 tools/codex/integration_coverage.py
```

## Saturating token budgets and checked lemmas

Codex's `approx_bytes_for_tokens` exposed a missing unsigned `saturating_mul`
model: neither its capped-product postcondition nor its panic obligation could
be discharged. The model now describes the mathematical product capped at the
integer maximum, matching the pinned Rust implementation's checked multiply.

`flux_core::num::lemmas` provides checked expansion and monotonicity facts with
explicit `requires` and `ensures`. Their bodies are verified under strict machine
integer bounds; they are not trusted declarations. Codex calls them under
`#[cfg(flux)]`, leaving ordinary builds free of proof calls. Expansion excludes a
zero multiplier, and monotonicity is non-strict because saturation can collapse
different inputs to the same result.

The regressions accept correct products for every unsigned primitive and reject
wrapping results, a wrong token multiplier, an invalid lemma call, and a false
strict-expansion lemma. Native checks exhaust all 65,536 byte-sized input pairs
and cover the saturation boundaries of wider types:

```sh
cargo x test saturating_mul
rustc --test tools/codex/model_checks/saturating_mul.rs -o /tmp/saturating-model-check
/tmp/saturating-model-check
```

The actual Codex caller and its monotonicity proof were checked individually.
Changing its multiplier from four to three failed both Flux checking and the
native budget regression. The correct version passed all 20 string-crate tests.

## Measuring incremental checks

Add `--proof-cache /path/to/cache` to library checks to reuse constraint results.
The cache namespace changes with the driver, library model metadata, fixpoint,
Rust version, solver version, and checking configuration. Within that namespace,
Flux validates the generated constraint hash for each query; editing source does
not discard unrelated queries. This is solver reuse, not incremental parsing or
a complete dependency proof system. The current experiment restricts caching to
library targets to avoid concurrent target invocations sharing a cache file.

Repeat `--only-check` to select several related bodies. Selection is attached to
the chosen package, so dependencies do not inherit a filter that skips their
checks. Each report includes available checker, body, and solver times, actual
body-cache hits, Cargo command duration, and observed build-lock contention.
Missing timing files remain missing evidence. Compiler invocation timing files
may overwrite each other for repeated crate names; all-target runs are therefore
not a reliable per-invocation performance benchmark yet.

The budget benchmark requires the Codex token-budget contract and proof:

```sh
python3 tools/codex/benchmark_incremental.py \
  --source /path/to/prepared-codex --flux "$PWD" \
  --output /tmp/codex-budget-benchmark
```

It runs three selected bodies with an empty cache, repeats unchanged checks, and
then mutates code, a contract, and a proof. Each mutation must fail, unaffected
queries must still be reused, and restored source must pass. Source and package
manifests are restored even on failure. Shared model changes invalidate the
namespace conservatively; selective transitive dependency invalidation remains
future work.

One development run measured 172 ms of checking with an empty cache and 20 ms
on each unchanged rerun (three cache hits, no solver queries). Code and proof
mutations reused two bodies; a contract mutation reused one. Cargo commands took
about 3 seconds on unchanged reruns and include work outside the verifier.
These are observations for three arithmetic bodies during development, not a
whole-project performance claim. Reported Cargo duration excludes corpus
inventory and evidence hashing; use process wall time for total harness cost.

## First pass through UTF-8 string splitting

Codex's original `split_string` loop now checks under strict overflow and
no-panic settings with a contract for prefix/suffix contents, individual byte
budgets, and combined retained length at most the input length. The Rust
implementation is unchanged. Its two local candidate invariants describe UTF-8
boundaries and the suffix budget; the solver must establish both.

The generalized models keep Rust byte length separate from SMT `str_len`, which
counts characters. `CharIndices` carries source text and a front byte offset;
its next item has valid start/end boundaries. Range indexing requires those
boundaries, not merely numeric bounds. The byte length and interior-boundary
functions remain uninterpreted, so these are partial external models, not a
proof of the Rust standard library. Ordinary, prefix, suffix, and full ranges
are modeled; inclusive ranges and exact character accounting remain future work.

A new default string-equality qualifier preserves iterator provenance across
loop joins. It proposes an invariant that is checked, rather than assuming
strings are equal. Positive/negative regressions verify guarded slicing and
reject wrong widths, arbitrary byte boundaries, inside-character slices, false
slice lengths, and false conclusions after exhausting an iterator.

The first-pass evidence is retained in
[`review/string-split-first-pass.json.gz`](review/string-split-first-pass.json.gz).
The full Flux suite passed 1,102 tests (7 ignored). Native checks compare the
width formula with every Unicode scalar value and exercise cursor/slice models.
Codex's 21 string-crate tests pass; an independent boundary oracle covers 971
input/budget combinations, including overlap, all UTF-8 widths, combining marks,
joined emoji, NUL, and near-maximum budgets.

Mutating the real implementation to retain too much prefix or suffix fails both
Flux and native tests. Doubling the removed-character counter fails the native
oracle but still passes Flux: **the current contract does not specify that
counter**. Exact removed counts and maximal retained slices are tested natively,
not formally proved by this first pass. All mutations are restored afterward.

```sh
rustc --test tools/codex/model_checks/utf8.rs -o /tmp/utf8-model-check
/tmp/utf8-model-check
python3 tools/codex/check_string_split.py \
  --source /path/to/prepared-codex --flux "$PWD" \
  --native-target /path/to/native-target \
  --output /tmp/codex-string-split-experiment
```

Use Codex's `spec/flux-string-split` branch, with `just` and `cargo-nextest` in the
pinned environment. The experiment checks the contract, runs the native oracle,
mutates the implementation, and verifies restored source. If the count contract
is strengthened later, update the intentionally recorded count-specification gap.

The cold split body took 622 ms, including 575 ms in the solver; the unchanged
body took 46 ms with one cached query and no solver work. Total reported checker
time did not improve (684 ms cold, 740 ms unchanged), and Cargo took 3.57/2.07 s.
Profile the work outside the body check before claiming an overall speedup.

Checking the full string library completes all 19 function/method checks: 8 are
accepted under the models and 11 report errors (52 diagnostics). The accepted
items include helpers without behavioral contracts, so this is not complete
verification of the crate. Next work is exact character accounting and maximality,
followed by the truncation callers, markers, allocation arithmetic, and public
output-budget semantics.

Model development also reduced a separate compiler crash: `Self` in an associated
refinement inside a generic extern impl for primitive `str` loses impl arguments
before Rust normalization. The models spell `str` explicitly. The unresolved
reproducer is `tests/tests/todo/primitive_self_projection.rs`; it is retained as
a blocker, outside the passing suite.
