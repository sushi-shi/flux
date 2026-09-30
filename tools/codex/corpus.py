#!/usr/bin/env python3
"""Reproduce Codex checks without treating a Cargo exit code as proof coverage."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
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


def check(source, flux, packages, output, offline, only_check=None):
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
                    if any("lib" in t["kind"] for t in p["targets"])]
    unknown = set(packages) - by_name.keys()
    if unknown:
        raise ValueError(f"Unknown packages: {sorted(unknown)}")
    if any(not any("lib" in t["kind"] for t in by_name[p]["targets"]) for p in packages):
        raise ValueError("This first runner checks library targets only; other targets remain unattempted")
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
        "-Ftimings=on", "-Fsummary=on",
    ]
    flux_patch = subprocess.check_output(["git", "diff", "--binary", "HEAD"], cwd=flux)
    (output / "flux.patch").write_bytes(flux_patch)
    report = {
        "schema_version": 1, "source": identity,
        "flux_revision": command(["git", "rev-parse", "HEAD"], flux),
        "flux_patch_sha256": hashlib.sha256(flux_patch).hexdigest(),
        "driver_sha256": file_digest(sysroot / "flux-driver"),
        "model_sha256": {p.name: file_digest(p) for p in sorted(sysroot.glob("*.fluxmeta"))},
        "fixpoint_sha256": file_digest(Path(shutil.which("fixpoint"))),
        "rustc": command(["rustc", "-Vv"]),
        "solver": command(["z3", "--version"]),
        "flags": flags,
        "only_check": only_check,
        "runner_sha256": file_digest(Path(__file__)),
        "configuration": "host target, default features, selected library bodies; tests and binaries not checked",
        "packages": [],
    }
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
        args = [str(binary), "flux", "check", "-p", name, "--lib", "--locked"]
        if only_check:
            args.extend(["--only-check", only_check])
        args.extend("-" + flag.replace("=on", "=true") for flag in flags)
        args.append("--Flog-dir=" + str(log_dir))
        if offline:
            args.append("--offline")
        start = time.monotonic()
        try:
            manifest.write_bytes(original + b"\n[package.metadata.flux]\nenabled = true\n")
            with (log_dir / "output.log").open("w") as log:
                result = subprocess.run(args, cwd=source / "codex-rs", env=env,
                                        stdout=log, stderr=subprocess.STDOUT)
        finally:
            manifest.write_bytes(original)
        outcome = classify(result.returncode, (log_dir / "output.log").read_text())
        outcome.update({
            "package": name, "command": args, "returncode": result.returncode,
            "elapsed_seconds": round(time.monotonic() - start, 3),
            "diagnostics": str(log_dir.relative_to(output) / "output.log"),
            "specification_status": "unmapped",
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
    run.add_argument("--only-check")
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
        report = check(args.source, args.flux.resolve(), args.package,
                       args.output.resolve(), args.offline, args.only_check)
        return int(any(p["status"] != "checked_with_observed_models" for p in report["packages"]))
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (ValueError, subprocess.CalledProcessError) as error:
        sys.exit(str(error))
