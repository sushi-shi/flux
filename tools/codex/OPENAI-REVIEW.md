# Codex as a working corpus for agent-assisted Rust verification

We are developing Flux against the real Codex codebase to test whether agents can
write readable contracts, library models, and reusable checked lemmas that make
review easier. The method is implementation-driven: check real code, reduce a
failure, fix or model the missing behavior, retain positive and negative tests,
then apply the improvement consistently to the next part of Codex.

This is a review checkpoint. The whole Codex tool is the target, but it is **not
fully specified or verified**. This package is prepared for a possible OpenAI
review; no review or endorsement is implied.

## Start here

- [Specification](../../handoff/SPECIFICATION.md): intended behavior, readability,
  incremental adoption, proof reuse, and completion criteria.
- [Corpus instructions](README.md): commands, evidence format, scope, and cache
  behavior.
- [Package worklist](review/baseline-summary.json): one outcome for each of 152
  packages, with recurring crash locations grouped for reduction.
- [Evidence manifest](review/manifest.json): checksums for the compressed function
  observations and diagnostics. Decompress with Python's standard `gzip` and
  `json` modules; absolute paths in diagnostics describe the original machine.
- [Codex contract stack](https://github.com/sushi-shi/codex/pull/2) and
  [Flux implementation stack](https://github.com/sushi-shi/flux/pull/12): draft PRs
  with focused changes and validation. Each PR targets the preceding branch.

## What the experiment has produced

Codex's string budget code now has an exact balanced-split contract, a capped
token-to-byte conversion contract, and a monotonicity proof. The implementation
uses shared checked lemmas for unsigned saturating multiplication. Their
`requires` and `ensures` are explicit; the lemmas are verified bodies, not trusted
axioms. Proof calls are compiled only under `cfg(flux)`.

For example, the balanced split contract says the two pieces use the entire
budget and differ by at most one:

```rust
#[cfg_attr(flux, flux::sig(fn(budget: usize[@b]) ->
    (usize[#prefix], usize{suffix:
        prefix + suffix == b && prefix <= suffix && suffix <= prefix + 1})))]
fn split_budget(budget: usize) -> (usize, usize) {
    let left = budget / 2;
    (left, budget - left)
}
```

The [shared arithmetic lemmas](../../lib/flux-core/src/num/lemmas.rs) state their
assumptions at the boundary. Expansion requires a positive multiplier;
monotonicity promises a non-strict inequality because saturation can produce the
same result for different inputs. These are examples for readability review,
not evidence that the current syntax is already ideal.

The unsigned multiplication model was checked against all 65,536 `u8` input pairs
and wider integer boundaries. Codex's string crate passed its 20 native tests.
Changing the actual token multiplier from four to three failed both native and
Flux checks. Separate code, contract, and proof mutations test that incremental
cache reuse still rejects changed obligations.

Applying the tool uncovered checker and model problems, not just missing Codex
annotations:

| Trigger | Generalized change | Draft PR |
| --- | --- | --- |
| Generic type layouts | Keep unavailable layout refinements symbolic | [#3](https://github.com/sushi-shi/flux/pull/3) |
| Associated-type constraints | Preserve sort identities while introducing holes | [#4](https://github.com/sushi-shi/flux/pull/4) |
| Filtered and infinite iterators | Require explicit capability for exact size models | [#5](https://github.com/sushi-shi/flux/pull/5) |
| Partial and interrupted checks | Journal functions, contracts, and outcomes | [#6](https://github.com/sushi-shi/flux/pull/6) |
| Workspace scale | Bound each package run and kill its process group on timeout | [#7](https://github.com/sushi-shi/flux/pull/7) |
| Token budgets | Model saturating multiplication and prove shared lemmas | [#8](https://github.com/sushi-shi/flux/pull/8) |
| String pattern guards | Exclude borrow-check-only temporaries from runtime checking | [#9](https://github.com/sushi-shi/flux/pull/9) |
| Repeated selected checks | Measure constraint reuse and invalidate changed queries | [#10](https://github.com/sushi-shi/flux/pull/10) |
| A closure inside a Codex loop | Rebuild nested shapes in the current parent scope | [#11](https://github.com/sushi-shi/flux/pull/11) |
| A false assertion in a directly called closure | Use the checked template at direct calls | [#12](https://github.com/sushi-shi/flux/pull/12) |

The last soundness bug was discovered by writing a negative regression for the
loop crash. The baseline checker accepted an assertion that clamping an arbitrary
integer to zero always produces a strictly positive result. The fixed checker
rejects it and proves the valid nonnegative result, including a vector built in
a loop. Regressions cover `Fn`, `FnMut`, `FnOnce`, captures, repeated calls, and
mutable arguments. The final full Flux suite passed 1,100 tests, with 7 ignored.

## Workspace baseline and its limits

The baseline pins Codex revision
`8346dbdde4424aa73743bf54cda773c165a310bb`, with the first budget contract. It uses
host Linux, default features, all available Cargo targets, strict overflow
checking, and no-panic obligations. Other platforms, features, and targets with
unmet required features remain outside this run. Targets are compiled, not
executed as a full Codex runtime test suite.

| Package outcome | Count |
| --- | ---: |
| Checker or compiler crash | 77 |
| Proof failure | 41 |
| Timeout | 29 |
| Other build/checker failure | 3 |
| Dependency build-script crash | 2 |
| Total | 152 |

The run collected 100,082 item observations across compiler invocations, including
duplicates, tests, generated code, and closures. This is **not a unique function
count or a proof-coverage percentage**. Closures are obligations of enclosing
bodies. Interrupted inventories and bodies remain explicitly incomplete.

One package initially encountered `Text file busy` during compiler startup while
the tool was being reinstalled. A retry with identical source, driver, model,
solver, and checking context supersedes that observation. Both reports and logs
are retained. The baseline timeout was 45 seconds per package; the retry used
120 seconds. Two V8 build-script panics are classified as dependency failures.

The baseline predates later model and checker fixes. It is retained as a worklist
and comparison point, not relabeled as current verification evidence. For
example, the loop-scope fix let `codex-agent-roles` complete 11 body checks before
an independent associated-sort crash, compared with 3 before that fix. Eight of
those 11 checks reported proof errors, and 38 bodies remained unattempted.

A later cross-package check completed all 47 function/method bodies in the
`codex-file-search` library without the previous scope crash: 8 were accepted
under the loaded models and 39 reported check errors. `codex-file-watcher` still
hits a scope assertion at an async lock's `.await`, where opaque sorts contain
different refinements between phases. The closure fix does not resolve that
case. Both observations are retained in `cross-package-evidence.json.gz`.

## Incremental performance

[Benchmark evidence](review/incremental-benchmark.json) records three actual Codex
budget bodies: `split_budget`, `approx_bytes_for_tokens`, and the monotonicity
proof. The benchmark starts with an empty proof cache, repeats unchanged checks,
then mutates code, a contract, and the proof. Every mutant must fail, unaffected
queries may be reused, and restored source must pass.

The retained review run took 72 ms in the checker with an empty cache and 16 ms
on each unchanged rerun, reusing all three body queries with no solver queries.
Cargo commands took 4.17 seconds initially and 1.32–1.42 seconds on unchanged
reruns. All three mutants failed and all restored versions passed. Exact timings,
per-body outcomes, and tool hashes are in the JSON; individual run reports,
source patches, and diagnostics are in `budget-run-evidence.json.gz`.

These are small-body measurements, not a whole-project speed claim. Cargo timing
excludes harness inventory and evidence hashing. Solver cache reuse does not
avoid Rust parsing or implement a complete dependency proof graph. Model, driver,
fixpoint, Rust, solver-version, or checking-configuration changes conservatively
invalidate the namespace. Source edits are invalidated by generated query hashes.
All-target timing files can overwrite repeated crate names, so they cannot yet
support reliable per-invocation performance comparisons.

## Questions for review

1. Do the Codex contracts express behavior a maintainer wants guaranteed, and are
   the requirements, postconditions, and lemmas readable enough to review?
2. Are the generalized Flux rules and external models sound? The negative tests
   are evidence for specific cases, not a general soundness proof.
3. Which Codex boundaries should get the next meaningful contracts: truncation,
   configuration validation, protocol/state transitions, or effectful tool calls?
4. What incremental CI feedback and latency would make this useful during review?

## Next implementation work

First reduce recurring crashes in associated-sort inference and type environments,
including the next agent-role loader failure and the file watcher's async opaque
sort mismatch. Preserve the remaining
`filter(...).enumerate()` escaping-bound-variable reproducer. Extend iterator
models only with justified preconditions and positive/negative tests.

Continue adding contracts to real Codex behavior and extracting reusable checked
lemmas. A package is not finished merely because its bodies are accepted under
weak default models. Track contract adequacy, model and trusted-code dependencies,
effect/state assumptions, and unsupported configuration paths. Full-project
completion requires meaningful specifications or explicit reviewed dispositions
throughout the pinned project, plus checks for the supported configuration matrix.

Measure where time goes before optimizing: Cargo rebuilds and lock waits, Rust
compilation, Flux shape/refinement checking, and solver queries. Keep mutation
tests alongside every cache or proof-reuse optimization. Fix per-invocation
timing attribution before making workspace throughput claims.

## Reproduce and inspect

Build the pinned environment and sysroot using the repository's handoff
instructions. Prepare an isolated Codex checkout; the runner restores temporary
manifest changes and never needs to modify the user's ordinary checkout.

```sh
python3 tools/codex/corpus.py prepare \
  --source /path/to/codex --revision 8346dbdde4424aa73743bf54cda773c165a310bb \
  --destination /tmp/codex-review-baseline
python3 tools/codex/corpus.py check \
  --source /tmp/codex-review-baseline --flux "$PWD" \
  --all-packages --targets all --timeout 45 --offline \
  --output /tmp/codex-review-baseline-results
python3 tools/codex/export_review.py \
  --baseline /tmp/codex-review-baseline-results \
  --output /tmp/codex-review-export
```

Offline checks require dependencies already present. The token-budget benchmark
needs Codex revision `09c7398ae0f62c7554ded40717f93d4ffab62515` from its second PR;
see the corpus instructions for that command. Exact historical baseline context,
source patches, model/driver hashes, compiler version, and solver version are
embedded in `codex-function-baseline.json.gz`. Running the latest stack produces
a new observation; it does not reproduce the old tool binaries byte for byte.
