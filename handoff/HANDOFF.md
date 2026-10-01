# Handoff: agent-oriented refinement types on Flux

State as of 2026-09-30. This directory lives on branch `handoff` of `sushi-shi/flux`, stacked on
`fix/iter-adapter-soundness` (PR #1). Everything needed to reproduce the results below is in here.

## Where things are

| What | Where | State |
|---|---|---|
| Soundness fixes | PR [sushi-shi/flux#1](https://github.com/sushi-shi/flux/pull/1), branch `fix/iter-adapter-soundness` | Open, waiting for review. Squash-merge (the first commit's neg test needs the second commit). |
| This handoff | branch `handoff` = PR #1 + `handoff/` | Not meant to be merged as is |
| Dev environment | `handoff/flake.nix` | Tested |
| Probes (small repros) | `handoff/probes/` | Results below were produced on this branch |
| Benchmarks | `handoff/bench/` | Tested |
| Codex triage template | `handoff/codex/` | Tested |
| Upstream base | `flux-rs/flux` at `efa0498` (2026-09-28) | The fork's `main` equals this commit |

## Goal and direction

We want a verifier for Rust where **agents write specs and proofs, and humans review only the specs and
the list of `trusted` items**. Writing proofs is tedious for humans but fine for agents, and checking a
proof is cheap and trustworthy. Decisions so far:

- Build on a **fork of Flux**: it runs on stock Rust, is fast, infers loop invariants, handles enums, and
  can export proofs to Lean. Don't start from scratch. Verus has the better proof story today but is a Rust
  dialect; it has not been tried hands-on yet, and trying it is the one thing that could change this decision.
- The desired surface (none of it is built yet):
  - specs written as plain Rust expressions (`#[requires]`/`#[ensures]`), including `matches!` on enum variants;
  - error contracts on `Result` ("fails only with `NotFound` or `Denied`");
  - any `const fn` usable in specs (refinement reflection, as in Liquid Haskell);
  - proofs written at call sites, and lemmas.
- Lemmas already exist in Flux as functions returning `()` with an `ensures` (see
  `tests/tests/pos/surface/bounded_quant00.rs`). Specs are shipped with crates (`flux-core` and `flux-alloc`
  are just spec crates), so Flux works as an extensible library.

## Environment (NixOS)

```sh
nix develop ./handoff              # pinned nightly + rustc-dev, z3, patched fixpoint, rustup/cargo/rustc shims
cargo x run FILE.rs                # dev build; single file
cargo x run FILE.rs -- -Fstd-extern-specs
cargo x test [FILTER]              # full suite: must end with 0 failed
```

Gotchas:
- **`-Fstd-extern-specs` is off by default.** Without it `Vec` indexing isn't checked at all, and
  "passes" on real code mean little. The test suite gets std specs through `extern crate flux_core;` in
  `tests/tests/with_deps/`.
- **cargo-flux assumes rustup** (`rustup which`, `cargo +toolchain`). The flake provides shims for these.
- **The `fixpoint` hash in the flake breaks** when upstream republishes the `nightly` tag. Re-run
  `nix-prefetch-url <url>` and update the hash.
- **Release builds:** `cargo x run` always builds in dev profile. `git apply
  handoff/bench/xtask-release-profile.patch`, then `XPROF=release cargo x run ...`. Dev timings are
  misleading.
- **The Rust port of fixpoint** is `cargo x --rust-fixpoint ...` (needs libclang, which the flake provides).
- **Extern specs must match std's impl headers exactly**, including generic parameter names. `impl
  Iterator for Bytes<'_>` can't be expressed yet (the `'_` lifetime).
- **Running on a cargo project:** add `[package.metadata.flux] enabled = true` to the crate, then run
  `FLUX_SYSROOT=<flux>/sysroot RUSTFLAGS="-L <flux>/sysroot" FLUXFLAGS="-Fstd-extern-specs -Ftimings"
  <flux>/target/debug/cargo-flux flux check -p <crate>`.

## Findings

### Soundness (the most important result so far)

Found by planting an off-by-one index in Codex's `utils/fuzzy-match` (it panics at runtime) and seeing
Flux accept it. Probe: `handoff/probes/soundness_iterators.rs` (run with `-- -Fstd-extern-specs`).

| Bug | Cause | Status |
|---|---|---|
| Every `for x in it.map(f)` body was vacuous, even over slices | `check_call` ignored the parent impl's `F: FnMut(..) -> B` clause. `B`'s kvar had no lower bound, so fixpoint solved it to `false` | **Fixed in PR #1** (`crates/flux-refineck/src/checker.rs`) |
| `s.chars().enumerate()` proved `ensures 1 == 2`; `count("ab".chars()) == count("abc".chars())` verified | The `Iterator` contract says `size` shrinks on each `Some`. `Chars` had no refinement (unit index), so the contract became `size(x) == size(x) - 1` | **Fixed in PR #1** for `Chars`, `CharIndices`, `Lines` (`lib/flux-core/src/str/`) |
| Same contradiction for any iterator type without a refinement (`Filter`, `Split`, `HashMap` iterators, …) | Same as above; the generic contract is assumed for impls that never promised it | **OPEN.** `count_filter_equal` in the probe is still accepted |
| `.enumerate()` over `split()` or a local custom iterator | Internal error `flux-infer/src/infer.rs:891` (crash, not unsound) | Open |
| `codex-mermaid` | Internal compiler error `flux-infer/src/infer.rs:488` "impossible case reached" | Open, not minimized |

Lesson: trusted specs are the weak point. One wrong spec made all the code after it verify silently. Any
spec library, especially one written by agents, needs **validation by execution**: run the real code on
generated inputs and check the spec's claims. That's how this bug was found: "Flux says safe, but the
program panics".

### Expressiveness (probes in `handoff/probes/`)

| Probe | Result |
|---|---|
| `liquid_vec_callee.rs` | "len > 5 and all elements == 5" is accepted by a callee requiring `len >= 1 && xs[0] == 5`. `len % 2 == 1` is correctly rejected, and accepted under `if xs.len() % 2 == 1`. Works only because the "all elements" part is in the element type (`RVec<i32{v: v == 5}>`); a runtime `.all()` check teaches Flux nothing. |
| `enum_variants.rs` | Enum variant refinements work: `unreachable!()` is proven unreachable, error-set contracts are checked, `e @ Error::NotFound(_)` carries the fact into the arm. Enums with fields need hand-numbered tags; `#[reflect]` only covers fieldless enums. |
| `arith.rs` | Nonlinear facts (`n*n > 25`, `n*(n+1)` even) and simple loop invariants are proven automatically. Overflow isn't checked by default. |
| `loop_element_inference.rs` | The element refinement of a vector that grows in a loop is **not** inferred (`with_loop` fails; `no_loop` passes). |
| `std_bounds.rs` | With std specs on, `Vec`/slice bounds are checked as expected. |

### Performance

- **Test suite, release build:** 422 files and 1,036 functions. Average about 10ms per function; per
  file, p50 7ms and p99 188ms; about 56ms per file including rustc.
- **Floor of 17–20ms per function** even for trivial code: `fixpoint` and z3 are started as separate
  processes for every function (z3 startup alone is about 12ms).
- **Loop-invariant inference in one function grows steeply**
  (`handoff/bench/gen_loops.sh K infer|manual`):

  | Loops | Inferred | Hand-written (helper functions with explicit signatures) |
  |---|---|---|
  | 8 | 0.48s | 0.23s |
  | 16 | 3.4s | 0.46s |
  | 32 | 39.8s | 1.14s |
  | 64 | >300s | 4.4s |

  Hand-written invariants make cost predictable, which fits "agents write the invariants".
- **The Rust port of fixpoint** (`--rust-fixpoint`) is 3× faster on typical files (p90 19ms vs 61ms)
  but not usable yet: it wrongly rejects 5 valid tests, takes about 4s on KMP/FFT, and took 41s on the
  4-loop generated file that hand-written invariants check in 0.13s with the Haskell solver.

### Codex (openai/codex, clone at `18194bf`)

- **Scale:** about 1M non-test lines, 127 crates. About 7.1k `.expect(`, 1.4k `.unwrap()`, 800 `panic!`,
  180 `unreachable!`, and thousands of index expressions. Full coverage is not the goal. Most sites are
  proven with no annotation; the rest become a ranked list, handled by a spec, a `trusted` marker, or a
  confirmed bug.
- **Crates tried** (`fuzzy-match`, `template`, `stream-parser`, `mermaid`, `string`): 58–600ms of Flux
  time per crate. **No Codex bugs found.** Every warning so far is safe code relying on facts Flux doesn't
  know. Examples: `char_indices` positions, `Utf8Error::valid_up_to`, the parser rejecting empty graphs,
  and `check_label` running at all 17 call sites of `mermaid::put_text` (`handoff/codex/mermaid_zero_width.rs`
  shows the triage).
- **Triage workflow that produces credible findings:**
  1. Flux flags a spot.
  2. Try to build a triggering input through the public API.
  3. Report a confirmed panic with its reproducer, or add a spec or `trusted` marker.

## Next steps (roughly in priority order)

1. **Review and squash-merge PR #1.** Consider sending both fixes upstream to `flux-rs/flux`, and open
   an issue for the general iterator hole.
2. **Close the general iterator hole.** Impls without a spec must not get the `Iterator` size contract
   for free. The sound default is "no size facts", since iterators can be infinite (`repeat`, `cycle`).
   Then write specs for the common adapters: `Filter` (each `Some` consumes at least one inner item),
   `Zip` (min), `Chain` (sum), `Rev`, `Peekable`.
3. **Specs for string byte boundaries.** Define `boundary(s, i) := i == 0 || i == len || !is_continuation(byte_at(s, i))`:
   - `&s[a..b]` requires `boundary(s, a) && boundary(s, b)`;
   - `char_indices`, `find` and `floor_char_boundary` ensure boundaries;
   - `is_char_boundary(i)` returns `bool[boundary(s, i)]`;
   - lemmas: ASCII byte at a boundary ⇒ next index is a boundary; `is_ascii` ⇒ every index is a boundary.

   This targets the most common text-handling panic, needs no checker changes, and has obvious targets in
   Codex's truncation code.
4. **Spec validation by execution.** A harness that runs trusted/extern specs against the real code on
   generated inputs.
5. **Codex triage run.** TUI layout arithmetic (underflow wraps silently in release builds),
   `utils/output-truncation`, `apply-patch`. Report only confirmed bugs, and track the hit rate.
6. **Async.** Basic `async fn`/`.await` already works (`tests/tests/pos/surface/async0*.rs`). First
   measure crashes and unsupported features on async-heavy Codex crates: `tokio::spawn`/`JoinHandle`,
   `select!`, boxed `dyn Future`, async trait methods. A type-level invariant on `Mutex<T>` acts as a
   lock invariant; facts relating state across tasks are out of scope for refinement types.
7. **Readable front end (partially implemented).** `flux_attrs::{requires, ensures}` now lower
   separate conditions to a single checked signature. They support scalar parameters, immutable
   strings/slices, `result`, length/emptiness, string prefix/suffix predicates, and qualified
   payload-free `matches!` alternatives on reflected enums. Arithmetic uses mathematical integers;
   division/remainder currently require a positive literal divisor. Stacked conditions are all
   checked, and unsupported expressions fail explicitly. Slice equality is rejected because the
   current slice model tracks length, not contents. Named field projections are supported when
   the aggregate's refinement fields use the same names and are tied to its Rust fields.
   Mutable receivers/parameters, `old(expr)`, pure `if/else`, standard Result discriminants,
   and Vec lengths are supported. Preservation of mutable fields must be stated explicitly.
   `#[refined]` now infers a nongeneric named struct's refinement record and field connections
   from scalar, named refined, Vec, and BTreeMap fields. Container equality compares the
   modeled properties only, not complete Rust contents. Native builds erase the attributes.
   Codex uses the macros through `flux_core` under `cfg_attr(flux, ...)`; removing those wrappers
   still needs ordinary-build dependency integration. Payload bindings/guards, nested patterns,
   indexing definedness, generic/async functions, const-fn
   reflection and dedicated proof blocks remain open. Do not treat this first subset as completion
   of the syntax examples in SPECIFICATION.md.
   The relay buffer case uses a weighted BTreeMap model to connect its byte counter to stored
   Vec lengths. See [collection model audit](COLLECTION-MODELS.md) for assumptions and limits.
8. **Performance.** Keep one solver process running instead of starting one per function. Measure
   `-Fcache` (per-function query cache) and parallel checking of functions.
