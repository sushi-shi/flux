#!/usr/bin/env bash
# Time Flux over its own positive test suite, one file at a time.
#
#   git apply handoff/bench/xtask-release-profile.patch    # lets `cargo x run` build a release sysroot
#   nix develop ./handoff -c handoff/bench/corpus.sh > corpus.tsv
#
# Env: LIMIT=N to only run the first N files; RUST_FIXPOINT=1 for the in-process Rust solver.
# Output columns: file, functions checked, flux time (as printed by -Ftimings), wall ms, #errors.
set -euo pipefail
cd "$(dirname "$0")/../.."

fp=()
[ "${RUST_FIXPOINT:-0}" = 1 ] && fp=(--rust-fixpoint)

# Build the release sysroot once and capture the exact driver invocation xtask uses.
echo 'fn main() {}' > /tmp/flux-bench-smoke.rs
driver_cmd=$(XPROF=release cargo x "${fp[@]}" run /tmp/flux-bench-smoke.rs 2>&1 |
  grep '^\$ .*flux-driver' | tail -1 | sed 's/^\$ //; s| /tmp/flux-bench-smoke.rs$||')
[ -n "$driver_cmd" ] || { echo "could not capture driver command (is the xtask patch applied?)" >&2; exit 1; }
export FLUX_SYSROOT="$PWD/sysroot"

cd tests/tests/pos
find . -name '*.rs' -not -path '*/auxiliary/*' | sort | head -n "${LIMIT:-100000}" | while read -r f; do
  t0=$(date +%s%N)
  out=$(timeout 120 $driver_cmd "$f" -Ftimings --out-dir /tmp/flux-bench-out 2>&1 || true)
  t1=$(date +%s%N)
  nf=$(echo "$out" | awk '/Functions checked/ {print $NF}')
  tot=$(echo "$out" | awk '/Total running time/ {print $NF}')
  ne=$(echo "$out" | grep -c '^error' || true)
  printf '%s\t%s\t%s\t%s\t%s\n' "$f" "${nf:-0}" "${tot:-NA}" "$(( (t1 - t0) / 1000000 ))" "$ne"
done
