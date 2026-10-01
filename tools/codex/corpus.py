#!/usr/bin/env python3
"""Reproduce Codex checks without treating a Cargo exit code as proof coverage."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import signal
import subprocess
import sys
import time


def command(args, cwd=None, env=None):
    return subprocess.check_output(args, cwd=cwd, env=env, text=True).strip()


def write_json(path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2) + "\n")


def file_digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def source_identity(source):
    revision = command(["git", "rev-parse", "HEAD"], source)
    untracked = command(["git", "ls-files", "--others", "--exclude-standard"], source)
    if untracked:
        raise ValueError("Commit or remove untracked source files before recording a run")
    patch = subprocess.check_output(["git", "diff", "--binary", "HEAD"], cwd=source)
    return {
        "revision": revision,
        "patch_sha256": hashlib.sha256(patch).hexdigest(),
        "modified": bool(patch),
    }, patch


def metadata(source):
    return json.loads(command([
        "cargo", "metadata", "--no-deps", "--offline", "--locked", "--format-version", "1",
        "--manifest-path", str(source / "codex-rs/Cargo.toml"),
    ]))


def inventory(source):
    identity, _ = source_identity(source)
    meta = metadata(source)
    members = set(meta["workspace_members"])
    packages = sorted(
        (p for p in meta["packages"] if p["id"] in members), key=lambda p: p["name"]
    )
    tracked = command(["git", "ls-files", "*.rs"], source).splitlines()
    roots = sorted(
        ((Path(p["manifest_path"]).parent, p["name"]) for p in packages),
        key=lambda pair: len(pair[0].parts), reverse=True,
    )
    files = {p["name"]: [] for p in packages}
    unassigned = []
    for relative in tracked:
        path = source / relative
        for root, name in roots:
            if path.is_relative_to(root):
                files[name].append(relative)
                break
        else:
            unassigned.append(relative)
    entries = []
    for package in packages:
        entries.append({
            "name": package["name"],
            "manifest": str(Path(package["manifest_path"]).relative_to(source)),
            "targets": [{
                "name": target["name"], "kind": target["kind"],
                "source": str(Path(target["src_path"]).relative_to(source)),
                "required_features": target.get("required-features", []),
            } for target in package["targets"]],
            "features": package["features"],
            "workspace_dependencies": sorted({
                dep["name"] for dep in package["dependencies"] if dep.get("path")
            }),
            "rust_files": files[package["name"]],
            "specification_status": "unmapped",
            "verification_status": "not_attempted",
        })
    return {
        "schema_version": 1, "source": identity,
        "scope": "Workspace package and tracked source inventory; not a function or proof map",
        "packages": entries, "unassigned_rust_files": unassigned,
    }


ANSI = re.compile(r"\x1b\[[0-9;]*m")
SUMMARY = re.compile(
    r"summary\. (\d+) functions processed: (\d+) checked; (\d+) trusted; (\d+) ignored"
)


def classify(returncode, log):
    clean = ANSI.sub("", log)
    summaries = [dict(zip(
        ("processed", "checked", "trusted", "ignored"), map(int, match)
    )) for match in SUMMARY.findall(clean)]
    text = clean.lower()
    if "internal compiler error" in text or "panicked at" in text:
        status = "checker_or_compiler_crash"
    elif returncode:
        # Retain raw diagnostics: this is triage, never a confirmed bug claim.
        if any(message in text for message in (
            "refinement type error", "postcondition cannot be proved",
            "type invariant may not hold",
            "may panic:", "arithmetic operation may overflow",
        )):
            status = "proof_failure"
        elif "unsupported" in text or "not supported" in text:
            status = "unsupported_or_build_failure"
        else:
            status = "build_or_checker_failure"
    elif summaries and any(s["checked"] for s in summaries):
        status = "checked_with_observed_models"
    else:
        status = "no_verification_evidence"
    return {
        "status": status, "summaries": summaries,
        "scope_note": "Summaries are compiler invocations, not unique functions or a proof of all behavior",
    }


def read_coverage(path):
    """Read partial journals without promoting missing results to proof success."""
    functions = {}
    start = None
    inventory_complete = False
    finish = None
    errors = []
    for number, line in enumerate(path.read_text().splitlines(), 1):
        try:
            event = json.loads(line)
            kind = event['event']
            if kind == 'start':
                if start is not None or event.get('schema_version') != 1:
                    raise ValueError('duplicate start or unsupported schema')
                start = event
            elif kind == 'function':
                if event['id'] in functions:
                    raise ValueError('duplicate function identity')
                functions[event['id']] = event
            elif kind == 'result':
                functions[event['id']]['status'] = event['status']
            elif kind == 'inventory_complete':
                inventory_complete = True
            elif kind == 'finish':
                finish = event
            else:
                raise ValueError('unknown coverage event')
        except (KeyError, TypeError, ValueError) as error:
            errors.append(f'line {number}: {error}')
    complete = bool(start and inventory_complete and finish is not None and not errors)
    entries = list(functions.values())
    counts = {}
    for function in entries:
        if function['status'] == 'in_progress':
            function['status'] = 'interrupted'
        status = function['status']
        counts[status] = counts.get(status, 0) + 1
    return {
        'journal': path.name, 'crate': start.get('crate') if start else None,
        'complete': complete, 'inventory_complete': inventory_complete,
        'crate_check_succeeded': finish.get('success') if finish else None,
        'errors': errors, 'counts': counts, 'functions': entries,
        'trust_dependencies': 'not_collected',
        'specification_status': 'needs_review',
    }


def function_map(log_dir):
    invocations = [read_coverage(p) for p in sorted(log_dir.glob('*-coverage.jsonl'))]
    return {
        'schema_version': 1,
        'scope': 'Active compiler configurations only; invocations are not deduplicated',
        'status': ('complete_observations' if invocations and all(i['complete'] for i in invocations)
                   else 'incomplete_or_unavailable'),
        'specification_status': 'needs_review',
        'invocations': invocations,
    }


def prepare(source, revision, destination):
    if destination.exists():
        raise ValueError("Destination already exists; refusing to overwrite it")
    revision = command(["git", "rev-parse", "--verify", revision + "^{commit}"], source)
    subprocess.run([
        "git", "clone", "--shared", "--no-checkout", str(source), str(destination),
    ], check=True)
    subprocess.run(["git", "checkout", "--detach", revision], cwd=destination, check=True)
    write_json(destination / ".git/flux-corpus.json", {
        "source": str(source), "initial_revision": revision,
    })


def execute_check(args, cwd, env, log, timeout):
    """Own a process group so timeout/interrupt also stops rustc and solver children."""
    def stop(process):
        try:
            os.killpg(process.pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            pass
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        process.wait()

    with subprocess.Popen(args, cwd=cwd, env=env, stdout=log, stderr=subprocess.STDOUT,
                          start_new_session=True) as process:
        try:
            return process.wait(timeout=timeout), False
        except subprocess.TimeoutExpired:
            stop(process)
            return process.returncode, True
        except BaseException:
            stop(process)
            raise


def proof_cache_namespace(report):
    # Source revision is deliberately excluded: changed constraints are invalidated
    # per query by Flux. Changing the tool, trusted models, solver, or check mode
    # invalidates the whole namespace conservatively.
    context = {key: report[key] for key in (
        'driver_sha256', 'model_sha256', 'fixpoint_sha256', 'rustc', 'solver',
        'flags', 'configuration',
    )}
    return hashlib.sha256(json.dumps(context, sort_keys=True).encode()).hexdigest()


def performance_evidence(log_dir):
    invocations = []
    for path in sorted(log_dir.glob('*-timings.json')):
        data = json.loads(path.read_text())
        invocations.append({
            'timings': path.name,
            'checker_ms': data['total'],
            'body_ms': sum(f['time_ms'] for f in data['functions']),
            'solver_ms': sum(q['time_ms'] for q in data['queries']),
            'executed_solver_queries': len(data['queries']),
            'cached_bodies': data.get('cached_bodies'),
        })
    return invocations


def check(source, flux, packages, output, offline, only_check=None, targets='lib', timeout=300,
          proof_cache=None):
    selections = [only_check] if isinstance(only_check, str) else list(only_check or [])
    if proof_cache is not None and targets != 'lib':
        raise ValueError('Proof-cache experiments currently require library targets')
    if timeout <= 0:
        raise ValueError("Timeout must be positive")
    if not (source / ".git/flux-corpus.json").is_file():
        raise ValueError("Use prepare first; checks only modify an owned corpus checkout")
    if output.exists():
        raise ValueError("Output already exists; use a new run directory to preserve evidence")
    if output.is_relative_to(source):
        raise ValueError("Keep reports outside the source checkout")
    baseline = inventory(source)
    by_name = {p["name"]: p for p in baseline["packages"]}
    if packages is None:
        packages = [p["name"] for p in baseline["packages"]
                    if targets == "all" or any("lib" in t["kind"] for t in p["targets"])]
    unknown = set(packages) - by_name.keys()
    if unknown:
        raise ValueError(f"Unknown packages: {sorted(unknown)}")
    if targets == "lib" and any(not any("lib" in t["kind"] for t in by_name[p]["targets"]) for p in packages):
        raise ValueError("Selected package has no library; use --targets all")
    sysroot = flux / "sysroot"
    binary = flux / "target/debug/cargo-flux"
    if not binary.is_file() or not (sysroot / "flux-driver").is_file():
        raise ValueError("Build Flux first: cargo x build-sysroot; cargo build -p flux-bin --bin cargo-flux")
    identity, patch = source_identity(source)
    output.mkdir(parents=True)
    (output / "codex.patch").write_bytes(patch)
    write_json(output / "inventory.json", baseline)
    flags = [
        "-Fstd-extern-specs=on", "-Fcheck-overflow=strict", "-Fno-panic=on",
        "-Ftimings=on", "-Fsummary=on", "-Fcoverage=on",
    ]
    flux_patch = subprocess.check_output(["git", "diff", "--binary", "HEAD"], cwd=flux)
    (output / "flux.patch").write_bytes(flux_patch)
    report = {
        "schema_version": 1, "source": identity,
        "flux_revision": command(["git", "rev-parse", "HEAD"], flux),
        "flux_patch_sha256": hashlib.sha256(flux_patch).hexdigest(),
        "driver_sha256": file_digest(sysroot / "flux-driver"),
        "cargo_flux_sha256": file_digest(binary),
        "model_sha256": {p.name: file_digest(p) for p in sorted(sysroot.glob("*.fluxmeta"))},
        "fixpoint_sha256": file_digest(Path(shutil.which("fixpoint"))),
        "rustc": command(["rustc", "-Vv"]),
        "solver": command(["z3", "--version"]),
        "flags": flags,
        "only_check": selections,
        "runner_sha256": file_digest(Path(__file__)),
        "configuration": {"platform": "host", "features": "default", "targets": targets},
        "timeout_seconds_per_package": timeout,
        "scope_note": "Other platforms and feature configurations remain unattempted; required-feature targets may be skipped by Cargo",
        "packages": [],
    }
    cache_namespace = proof_cache_namespace(report) if proof_cache else None
    report['proof_cache'] = {'namespace': cache_namespace, 'root': str(proof_cache)} if proof_cache else None
    env = os.environ.copy()
    for key in ("RUSTC", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "RUSTFLAGS", "FLUXFLAGS"):
        env.pop(key, None)
    env.update({
        "FLUX_SYSROOT": str(sysroot), "CARGO_ENCODED_RUSTFLAGS": "-L\x1f" + str(sysroot),
        "CARGO_TARGET_DIR": str(output.parent / "target"), "CARGO_TERM_COLOR": "never",
    })
    for name in packages:
        manifest = source / by_name[name]["manifest"]
        original = manifest.read_bytes()
        if "[package.metadata.flux]" in original.decode():
            raise ValueError(f"Existing Flux metadata in {manifest}; refusing to rewrite it")
        log_dir = output / name
        log_dir.mkdir()
        # A unique logging flag forces Cargo to invoke Flux again even with warm artifacts.
        args = [str(binary), "flux", "check", "-p", name,
                "--lib" if targets == "lib" else "--all-targets", "--locked"]
        # Put selection on this package's metadata only; other enabled dependencies
        # must not inherit a command-line include filter and silently lose checks.
        flux_metadata = '\n[package.metadata.flux]\nenabled = true\n'
        if selections:
            flux_metadata += 'include = ' + json.dumps(selections) + '\n'
        cache_path = None
        if proof_cache:
            cache_path = proof_cache / cache_namespace / (name + '.json')
            args.append('--Fcache=' + str(cache_path))
        cache_existed = cache_path.is_file() if cache_path else False
        args.extend("-" + flag.replace("=on", "=true") for flag in flags)
        args.append("--Flog-dir=" + str(log_dir))
        if offline:
            args.append("--offline")
        start = time.monotonic()
        try:
            manifest.write_bytes(original + flux_metadata.encode())
            with (log_dir / "output.log").open("w") as log:
                returncode, timed_out = execute_check(
                    args, source / "codex-rs", env, log, timeout)
        finally:
            manifest.write_bytes(original)
        mapping = function_map(log_dir)
        write_json(log_dir / "function-map.json", mapping)
        outcome = classify(returncode, (log_dir / "output.log").read_text())
        if timed_out:
            outcome['status'] = 'timeout'
        outcome.update({
            "package": name, "command": args, "returncode": returncode, "timed_out": timed_out,
            "elapsed_seconds": round(time.monotonic() - start, 3),
            "diagnostics": str(log_dir.relative_to(output) / "output.log"),
            "specification_status": "needs_review",
            "function_map": str(log_dir.relative_to(output) / "function-map.json"),
            "mapping_status": mapping['status'],
            "dependency_proofs": "not_collected",
            "temporary_flux_metadata": flux_metadata,
            "cache_existed_before_run": cache_existed,
            "performance": performance_evidence(log_dir),
            "build_lock_wait_observed": 'Blocking waiting for file lock' in (log_dir / 'output.log').read_text(),
        })
        report["packages"].append(outcome)
        write_json(output / "report.json", report)
        print(f"{name}: {outcome['status']} ({outcome['elapsed_seconds']}s)", flush=True)
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    subs = parser.add_subparsers(dest="action", required=True)
    inv = subs.add_parser("inventory")
    inv.add_argument("--source", type=Path, required=True)
    inv.add_argument("--output", type=Path, required=True)
    prep = subs.add_parser("prepare")
    prep.add_argument("--source", type=Path, required=True)
    prep.add_argument("--revision", required=True)
    prep.add_argument("--destination", type=Path, required=True)
    run = subs.add_parser("check")
    run.add_argument("--source", type=Path, required=True)
    run.add_argument("--flux", type=Path, required=True)
    selection = run.add_mutually_exclusive_group(required=True)
    selection.add_argument("--package", action="append")
    selection.add_argument("--all-libraries", action="store_true")
    selection.add_argument("--all-packages", action="store_true")
    run.add_argument("--targets", choices=("lib", "all"), default="lib")
    run.add_argument("--timeout", type=float, default=300, help="Seconds per package, including compilation")
    run.add_argument("--only-check", action="append")
    run.add_argument("--proof-cache", type=Path, help="Reuse constraint queries in a tool/model/configuration namespace")
    run.add_argument("--output", type=Path, required=True)
    run.add_argument("--offline", action="store_true")
    args = parser.parse_args()
    args.source = args.source.resolve()
    if args.action == "inventory":
        result = inventory(args.source)
        write_json(args.output, result)
        print(f"{len(result['packages'])} packages inventoried; all initially unmapped")
    elif args.action == "prepare":
        prepare(args.source, args.revision, args.destination.resolve())
    else:
        if args.all_libraries and args.targets != "lib":
            parser.error("Use --all-packages with --targets all")
        if args.all_packages and args.targets != "all":
            parser.error("--all-packages requires --targets all")
        report = check(args.source, args.flux.resolve(), args.package,
                       args.output.resolve(), args.offline, args.only_check, args.targets, args.timeout,
                       args.proof_cache.resolve() if args.proof_cache else None)
        return int(any(p["status"] != "checked_with_observed_models" for p in report["packages"]))
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (ValueError, subprocess.CalledProcessError) as error:
        sys.exit(str(error))
