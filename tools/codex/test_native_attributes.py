"""Reject native erasing macros under verification; run inside the Flux dev shell."""

import json
import os
from pathlib import Path
import subprocess
import tempfile


def main():
    root = Path(__file__).resolve().parents[2]
    env = os.environ.copy()
    for key in ("FLUX_BUILD_SYSROOT", "CARGO_ENCODED_RUSTFLAGS", "RUSTFLAGS"):
        env.pop(key, None)
    message = "native Flux attributes cannot be used for verification"
    result = subprocess.run(
        ["cargo", "check", "-p", "flux-attrs", "--test", "readable_erasure"],
        cwd=root, env=env | {"RUSTFLAGS": "--cfg flux"},
        capture_output=True, text=True,
    )
    assert result.returncode != 0 and message in result.stderr, result.stderr

    build = subprocess.run(
        ["cargo", "build", "-p", "flux-attrs", "--message-format=json"],
        cwd=root, env=env, capture_output=True, text=True, check=True,
    )
    artifact = None
    for line in build.stdout.splitlines():
        data = json.loads(line)
        if data.get("reason") == "compiler-artifact" and data["target"]["name"] == "flux_attrs":
            artifact = next(p for p in data["filenames"] if p.endswith((".so", ".dylib", ".dll")))
    assert artifact, build.stdout
    with tempfile.TemporaryDirectory(prefix="flux-native-attributes-") as directory:
        source = Path(directory) / "false_contract.rs"
        source.write_text(
            "use flux_attrs::ensures;\n"
            "#[ensures(result == value + 1)]\n"
            "pub fn false_claim(value: u32) -> u32 { value }\n"
        )
        result = subprocess.run(
            [str(root / "sysroot/flux-driver"), str(source), "--crate-type=rlib",
             "--edition=2021", "-L", str(root / "sysroot"), "--extern", "flux_attrs=" + artifact,
             "-Fsysroot=" + str(root / "sysroot"), "-Fstd-extern-specs=on", "-Fverify=on",
             "--emit=metadata", "-o", str(Path(directory) / "false_contract.rmeta")],
            cwd=root, env=env, capture_output=True, text=True,
        )
        assert result.returncode != 0 and message in result.stderr, result.stderr
    print("Native macro misuse rejected by rustc and flux-driver")


if __name__ == "__main__":
    main()
