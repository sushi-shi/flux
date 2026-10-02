# Resume checkpoint: Flux applied to Codex

The user requested a deliberate stop after completing the current change, preserving a handoff, and removing build targets and redundant experiment checkouts. Resume only when requested. The full goal remains checking Codex with inferred summaries, useful contracts, models, lemmas and inline proofs; the project is far from fully verified.

## Repositories and stacks

- Flux: `https://github.com/sushi-shi/flux`, branch `feat/string-slice-drain`, draft [PR #30](https://github.com/sushi-shi/flux/pull/30), based on `feat/string-field-boundaries` ([#29](https://github.com/sushi-shi/flux/pull/29)). The tested compiler/model commit is `ef5241b9501554f1f4d95a676bdb0e3d4495f898`; later commits on this branch only document the checkpoint.
- Codex: `https://github.com/sushi-shi/codex`, branch `spec/flux-parser-slices`, draft [PR #19](https://github.com/sushi-shi/codex/pull/19), based on `spec/flux-parser-positions` ([#18](https://github.com/sushi-shi/codex/pull/18)). The new branch adds a verified suffix-start boundary contract and the evidence/handoff. Check `tools/flux/parser-slices.json` for exact revisions and results.
- Retained local checkouts: `/home/sheep/Projects/flux` and `/home/sheep/Projects/codex-flux`. The original `/home/sheep/Projects/codex` contains unrelated untracked planning files and is preserved. Flowistry is a separate project; its worktrees and changes are preserved.
- Both forks contain the prior stacked draft PRs. Continue stacking on the current branches; do not merge or rewrite published history. Sending anything upstream to OpenAI still requires a separate instruction.

## Working policy

Read each repository's `AGENTS.md`. Start from the implementation and facts a caller needs. Try inference, then generate and check proof support. Retain explicit requirements for stable intent and real abstraction gaps. Describe useful results; state preservation or absence only for a concrete caller, regression or interface reason. Generated proofs must establish their conclusions, and regenerated summaries must not silently replace accepted requirements.

## Completed implementation

Shared references now carry the logical value of their referents through generic boundaries, allowing a `Pattern` implemented by `&str` to describe the specific delimiter passed to `find` or `ends_with`. Mutable reference indices retain their previous representation.

Reference and referent indices are linked consistently at both type and generic-constructor creation, before inference holes become predicates. Canonicalization eliminates redundant existential indices, preserving borrowed-slice element facts at loop joins. Promoted-constant templates are also canonicalized before inference, keeping shape and refinement scopes consistent. Binder remapping subtracts local binder depth before consulting the outer binder mapping; the new reference representation exposed that bug during the model build.

String indexing delegates to the existing `SliceIndex<str>` boundary and output models. Prefix draining requires a valid UTF-8 endpoint and admits either the original contents or the remaining suffix. The pinned Rust implementation removes text in `Drain::drop`; forgetting the iterator leaves the original text. Exact removal must not be assumed at the call to `drain`.

Literal string search models establish that a match ends at a UTF-8 boundary and that a matching suffix starts at a boundary. Codex's existing `longest_suffix_prefix_len` body proves:

```rust
#[ensures(s.is_char_boundary(s.len() - result))]
```

The caller needs this to split pending text before the retained suffix. Runtime parser statements are unchanged.

## Validation and evidence

- Full Flux suite: **1,174 passed, 7 ignored** (`cargo x test`). This includes positive and negative reference/search/slice/drain tests, the existing borrowed-slice loop test, and the weighted-map negative test that exposed the promoted-reference scope mismatch.
- Native Codex stream parser: **30 passed** (`just test -p codex-utils-stream-parser`).
- Pinned Rust model audit: **96 checks passed**, covering empty/Unicode strings, literal search ends, suffix starts, and normal/partial/forgotten drains. Source: `tools/codex/model_checks/string_search_drain.rs`.
- The exact clean model revision was rebuilt for the Codex corpus checks. Codex's macro dependency stays at `d2b7ea21d4177217e10e93ebff162bbec9da9f19`; this step changes compiler/models, not macro syntax.
- Codex's `tools/flux/parser-slices.json` and accompanying compressed evidence are authoritative for selected proofs, the broader parser frontier, negative mutation, timings and source hashes. Earlier evidence remains in `tools/flux/`.

The final five-package sweep passed the selected checks in string, image, history and exec-server. The parser reported exactly four refinement errors across two functions: slicing and draining at `take` in `drain_visible_to_suffix_match`, and the boundary/range of `open_idx + open_len` in `push_str`. Closing-tag drains and the active-tag suffix split now discharge their obligations. Removing the helper's `ends_with` guard is rejected, and the mutation was restored.

The broader parser sweep took **139.790 seconds**, with **130.937 seconds in solver work** (11 executed queries and one already-cached helper). This is slower than the previous 56.211-second check with weaker models. Record it as a performance regression to investigate, not a speedup. The suffix helper itself verifies quickly and its repeated check executes zero solver queries.

Standard-library extern specifications remain trusted assumptions, audited against pinned Rust `8925ea358a0f265ca61026aadc7ecc506c545cbe` (nightly-2026-08-21). Native examples support that audit; they are not proofs of the standard library.

## Next work

Continue from the newly exposed parser obligations. Do not weaken or disable the new models to restore the previous green result.

1. Give `drain_visible_to_suffix_match` the boundary requirement its caller must satisfy, and propagate the verified suffix-start property through `max_open_prefix_suffix_len`. The iterator `Map` and `max` models need to carry the selected item's predicate. Readable `String` field `len()` is still unsupported; the current field-length lowering assumes `Vec`, so improve that abstraction if the required contract needs it.
2. Relate the selected opening-tag index to its delimiter value and prove the opening drain endpoint. The current Vec model carries length, not an index-to-element-value relation. The search contract currently exports the opening position and vector index bounds, but not the complete matched span.
3. Model owner effects of dropping a drain iterator to establish exact emitted/retained contents while preserving soundness for leaks. The current disjunction is intentionally insufficient for a full streaming interpretation theorem.
4. Measure and reduce the cost of the nested iterator/callback proof. Cache timings are evidence about selected checks with warm Cargo artifacts, not whole-project throughput.
5. Expand through actual callers and more Codex packages. The broader objective still includes complete streaming interpretation, useful reusable lemmas, readable syntax, incremental checking, models for effects/async/unsafe code, and reviewable evidence for OpenAI.

## Rebuild and resume

Build targets are removed at the user's request; expect a rebuild. Use **two Cargo jobs**. Do not rebuild the Flux sysroot while corpus checks run.

```sh
cd /home/sheep/Projects/flux
CARGO_BUILD_JOBS=2 nix develop ./handoff --command cargo x build-sysroot
CARGO_BUILD_JOBS=2 nix develop ./handoff --command cargo x test

CARGO_BUILD_JOBS=2 nix develop ./handoff --command python3 tools/codex/corpus.py check \
  --source /home/sheep/Projects/codex-flux --flux /home/sheep/Projects/flux \
  --package codex-utils-stream-parser --timeout 300 --offline \
  --only-check 'def:longest_suffix_prefix_len' \
  --output /tmp/flux-codex-results/resumed-suffix
```

To investigate the next obligations, select `def:InlineHiddenTagParser::<T>::drain_visible_to_suffix_match` and `def:InlineHiddenTagParser<T> as stream_text::StreamTextParser>::push_str`. Distinguish proof failures, unsupported features, crashes and timeouts. Poll an existing live process handle rather than restarting a check because observation timed out.

For native Codex checks use the repository's `just fmt` and `just test`, with `just`, `cargo-nextest`, `uv`, `dotslash` and `patchelf` available through the pinned Nix environment. Do not invoke `cargo test` directly in Codex. A full Codex test suite still needs user approval. Dependency changes require `just bazel-lock-update`; this last Codex change does not change dependencies.

The older `HANDOFF.md` records the initial experiment and historical findings. Its dated open-issue statuses do not supersede this checkpoint or the current evidence.

## Completed cleanup

The [cleanup record](CLEANUP.json) lists the removed paths and the pushed heads verified before deletion. Eight build-target directories (roughly 82 GB), two target symlinks, two redundant Flux worktrees, two baseline Codex clones, and the stale generated Flux sysroot were removed. Both retained repositories are clean and pushed. The relocated Codex repository passed a Git connectivity check after its former object sources were removed. No verifier or native-test process remains running.
