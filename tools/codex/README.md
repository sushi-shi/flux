# Codex verification corpus

Run real Codex code, reduce failures, add a general model, lemma, or checker fix,
and rerun the affected code. Expand this loop until the whole tool has meaningful
contracts and accounted-for proof obligations. The package inventory keeps work
that has not yet been attempted visible; it is not a function or specification map.

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
selection with `--all-libraries`. To reduce a failure, use
`--only-check 'def:function_name'`; the report records this restriction.

The initial configuration enables library models, strict arithmetic checking,
and panic obligations on the host platform with default features. Only library
targets are run by this harness; binaries, tests, feature combinations, and other
platforms remain explicit inventory entries, not verified coverage. General
iterator soundness gaps from the handoff are still open, so an accepted check is
only an observation under the loaded models, not a correctness certification.

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
