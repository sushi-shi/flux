# Dev shell for building and running Flux on NixOS: `nix develop ./handoff`
{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  inputs.rust-overlay.url = "github:oxalica/rust-overlay";
  inputs.rust-overlay.inputs.nixpkgs.follows = "nixpkgs";

  outputs = { nixpkgs, rust-overlay, ... }:
    let
      pkgs = import nixpkgs {
        system = "x86_64-linux";
        overlays = [ rust-overlay.overlays.default ];
      };

      # Keep in sync with `channel` in ../rust-toolchain.toml.
      tc = pkgs.rust-bin.nightly."2026-08-21".default.override {
        extensions = [ "rust-src" "rustc-dev" "llvm-tools" "rustfmt" "clippy" ];
      };

      # Prebuilt liquid-fixpoint (what upstream's install.sh uses). The `nightly` tag moves, so
      # this hash breaks when upstream republishes: re-run `nix-prefetch-url <url>` and update it.
      fixpoint = pkgs.stdenv.mkDerivation {
        pname = "liquid-fixpoint-bin";
        version = "nightly";
        src = pkgs.fetchurl {
          url = "https://github.com/ucsd-progsys/liquid-fixpoint/releases/download/nightly/fixpoint-x86_64-linux-gnu.tar.gz";
          sha256 = "19wd3lqp5ibk7n11v61xi633rnispnd3wfmq9v6lpw4s53qyvl6a";
        };
        sourceRoot = ".";
        nativeBuildInputs = [ pkgs.autoPatchelfHook ];
        buildInputs = [ pkgs.gmp ];
        installPhase = "install -Dm755 fixpoint $out/bin/fixpoint";
      };

      # cargo-flux assumes rustup: it runs `rustup which --toolchain <tc> <bin>` and
      # `cargo +<tc> ...` / `rustc +<tc> ...`. Answer those from the pinned nix toolchain.
      shims = pkgs.symlinkJoin {
        name = "flux-rustup-shims";
        paths = [
          (pkgs.writeShellScriptBin "rustup" ''
            [ "$1" = which ] || { echo "rustup shim: only 'rustup which' is supported" >&2; exit 1; }
            echo ${tc}/bin/''${@: -1}
          '')
          (pkgs.writeShellScriptBin "cargo" ''
            case "$1" in +*) shift ;; esac
            exec ${tc}/bin/cargo "$@"
          '')
          (pkgs.writeShellScriptBin "rustc" ''
            case "$1" in +*) shift ;; esac
            exec ${tc}/bin/rustc "$@"
          '')
        ];
      };
    in
    {
      devShells.x86_64-linux.default = pkgs.mkShell {
        packages = [ tc fixpoint pkgs.python3 pkgs.z3 pkgs.z3.dev pkgs.gmp pkgs.pkg-config pkgs.openssl pkgs.clang ];
        # Only needed for `cargo x --rust-fixpoint ...` (bindgen for z3-sys).
        LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
        BINDGEN_EXTRA_CLANG_ARGS = "-isystem ${pkgs.z3.dev}/include";
        shellHook = ''
          export PATH=${shims}/bin:$PATH
        '';
      };
    };
}
