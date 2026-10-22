# csan's flake: the `csan` command-line binary, built with the toolchain
# rust-toolchain.toml pins -- rust-overlay reads that file, so the MSRV is
# written once and rustup and Nix build with the same compiler -- the tools
# `just checks` and `just release` run, and CI's checks as derivations.
#
#   nix build            the binary, as ./result/bin/csan
#   nix run . -- --help  the binary, run (`nix run github:veralvx/csan -- ...`)
#   nix develop          the shell: the toolchain (with the wasm target), just,
#                        dprint, cog, git-cliff, cargo-audit, and the bindings'
#                        tools (maturin, wasm-pack, node, typst)
#   nix develop .#miri   nightly with Miri, for `just miri`
#   nix fmt              nixfmt, through treefmt; `nix fmt -- --ci` formats,
#                        and fails if that changed a file
#   nix flake check      clippy, rustfmt, rustdoc and the binary's tests, the
#                        format (`nix fmt`'s verdict) and both dev shells,
#                        offline in the sandbox; CI runs it on every system
#                        below (.github/workflows/nix.yml)
#
# The package is the CLI alone (crates/csan, with crates/csan-core): the
# Python, npm and Typst bindings are built by their own tools (see the
# justfile), and released by the Release workflow.
#
# flake.lock pins nixpkgs and rust-overlay: commit it; `nix flake update`
# moves it, and Dependabot does weekly (.github/dependabot.yml).
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { nixpkgs, rust-overlay, ... }:
    let
      inherit (nixpkgs) lib;
      # the Release workflow's targets, less Windows and Intel macOS:
      # nixpkgs dropped x86_64-darwin with 26.11 (the release archives and
      # installers still cover it)
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];

      # version, licence and repository from the manifests, so a release's
      # bump (`just release`) is the flake's
      manifest = lib.importTOML ./Cargo.toml;
      inherit (manifest.workspace.package) version license repository;
      cargoPackage = (lib.importTOML ./crates/csan/Cargo.toml).package;
      pname = cargoPackage.name;

      # what a build reads: the repository less its tooling and contributor
      # docs, so editing those rebuilds nothing, while a new crate or test is
      # in without a word
      src = lib.fileset.toSource {
        root = ./.;
        fileset = lib.fileset.difference ./. (
          lib.fileset.unions (
            map lib.fileset.maybeMissing [
              ./.github
              ./.gitattributes
              ./.gitignore
              ./CHANGELOG.md
              ./CODE_OF_CONDUCT.md
              ./CONTRIBUTING.md
              ./SECURITY.md
              ./cliff.toml
              ./cog.toml
              ./dist-workspace.toml
              ./dprint.json
              ./flake.lock
              ./flake.nix
              ./justfile
            ]
          )
        );
      };

      perSystem =
        pkgs:
        let
          rust-bin = rust-overlay.lib.mkRustBin { } pkgs;
          toolchain = rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          rustPlatform = pkgs.makeRustPlatform {
            cargo = toolchain;
            rustc = toolchain;
          };
          # `nix fmt`: the Nix. The Rust is rustfmt's (`just fmt`), the
          # Markdown, TOML and JSON dprint's.
          formatter = pkgs.treefmt.withConfig {
            settings = {
              on-unmatched = "info";
              formatter.nixfmt = {
                command = "nixfmt";
                includes = [ "*.nix" ];
              };
            };
            runtimeInputs = [ pkgs.nixfmt ];
          };
          common = {
            inherit version src;
            cargoLock.lockFile = ./Cargo.lock;
          };
          # a check: one cargo command over the whole workspace, offline
          # against the vendored lock, a stamp for output. The Python
          # bindings' build script (pyo3) needs an interpreter to configure
          # against.
          cargoCheck =
            name: command:
            rustPlatform.buildRustPackage (
              common
              // {
                pname = "${pname}-${name}";
                nativeBuildInputs = [ pkgs.python3 ];
                buildPhase = ''
                  runHook preBuild
                  ${command}
                  runHook postBuild
                '';
                doCheck = false;
                installPhase = ''
                  runHook preInstall
                  touch $out
                  runHook postInstall
                '';
              }
            );
          # the release build (Cargo.toml's profile: fat LTO, panic =
          # "abort") of the CLI, its own and its library's tests run by
          # buildRustPackage's checkPhase
          package = rustPlatform.buildRustPackage (
            common
            // {
              inherit pname;
              cargoBuildFlags = [ "--package=${pname}" ];
              cargoTestFlags = [
                "--package=${pname}"
                "--package=csan-core"
              ];
              meta = {
                inherit (cargoPackage) description;
                homepage = repository;
                # the SPDX expression's licence identifiers: its operators,
                # parentheses and `WITH` exceptions aside
                license = map lib.getLicenseFromSpdxId (
                  lib.subtractLists [ "" "AND" "OR" ] (
                    lib.filter lib.isString (builtins.split "[ ()]+|WITH [^ ()]+" license)
                  )
                );
                platforms = systems;
                mainProgram = pname;
              };
            }
          );
          devShells = {
            default = pkgs.mkShell {
              packages = [
                # the pinned toolchain, plus the target the Typst plugin and
                # the npm package compile to
                (toolchain.override { targets = [ "wasm32-unknown-unknown" ]; })
                pkgs.just
                pkgs.jq
                pkgs.dprint
                pkgs.cocogitto
                pkgs.git-cliff
                pkgs.cargo-audit
                pkgs.cargo-dist
                pkgs.nixfmt
                # the bindings: Python (maturin, pytest), npm (wasm-pack,
                # node, binaryen's wasm-opt) and the Typst package
                pkgs.maturin
                (pkgs.python3.withPackages (python: [ python.pytest ]))
                pkgs.wasm-pack
                pkgs.binaryen
                pkgs.nodejs
                pkgs.typst
              ];
            };
            # Miri runs on nightly only: rust-overlay's latest nightly that
            # ships it
            miri = pkgs.mkShell {
              packages = [
                (rust-bin.selectLatestNightlyWith (
                  nightly:
                  nightly.minimal.override {
                    extensions = [
                      "miri"
                      "rust-src"
                    ];
                  }
                ))
              ];
            };
          };
        in
        {
          packages.default = package;
          checks = {
            clippy = cargoCheck "clippy" "cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings";
            fmt = cargoCheck "fmt" "cargo fmt --all -- --check";
            doc = cargoCheck "doc" "RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --all-features --locked --offline";
            # the formatter's verdict on the whole tree: what `nix fmt` would
            # change fails it
            format = formatter.check ./.;
            inherit package;
            # the dev shells built, not just evaluated: an input update that
            # breaks a tool in either fails here, before Dependabot's pull
            # request can merge
            devShell = devShells.default;
            miriShell = devShells.miri;
          };
          inherit devShells formatter;
        };

      bySystem = lib.genAttrs systems (system: perSystem nixpkgs.legacyPackages.${system});
      output = name: lib.mapAttrs (_: outputs: outputs.${name}) bySystem;
    in
    {
      packages = output "packages";
      checks = output "checks";
      devShells = output "devShells";
      formatter = output "formatter";
    };
}
