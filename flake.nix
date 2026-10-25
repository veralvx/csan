# csan's flake: the `csan` command-line binary, built with the toolchain
# rust-toolchain.toml pins -- rust-overlay reads that file, so the MSRV is
# written once and rustup and Nix build with the same compiler -- the tools
# `just checks` and `just release` run, and CI's checks as derivations.
#
#   nix build            the binary, as ./result/bin/csan
#   nix run . -- --help  the binary, run (`nix run github:veralvx/csan -- ...`)
#   nix develop          the shell: the toolchain (with the wasm target), just,
#                        dprint, cog, git-cliff, cargo-audit, and the bindings'
#                        tools (maturin, pytest, mypy, ruby with bundler and
#                        libclang, wasm-pack, node, typst)
#   nix develop .#miri   nightly with Miri, for `just miri`
#   nix fmt              nixfmt, through treefmt; `nix fmt -- --ci` formats,
#                        and fails if that changed a file
#   nix flake check      clippy, rustfmt, rustdoc and the binary's tests, the
#                        format (`nix fmt`'s verdict) and both dev shells;
#                        the Typst package's harness (crates/csan-typst):
#                        its tools' pins, its tests (tytanic), its checks
#                        (`just typst ...`: the bundle, typst-package-check,
#                        the README's and the examples' Typst, the figures,
#                        tinymist's lint) and its justfile's shell; and the
#                        workflows (actionlint) -- offline in the sandbox;
#                        CI runs it on every system below
#                        (.github/workflows/nix.yml)
#
# The package is the CLI alone (crates/csan, with crates/csan-core): the
# Python, Ruby, npm and Typst bindings are built by their own tools (see the
# justfile), and released by the Release workflow.
#
# flake.lock pins nixpkgs, rust-overlay and typst-package-check: commit it;
# `nix flake update` moves it, and Dependabot does weekly
# (.github/dependabot.yml). The Typst harness's tools are pinned in
# crates/csan-typst/tools.toml too: after an update, the `typst-pins` check
# names each one that moved, for tools.toml.
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    # typst/packages' own checker, at its tag: nixpkgs has no build of it
    package-check = {
      url = "github:typst/package-check/v0.7.0";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      package-check,
      ...
    }:
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
      # the Typst harness's tools, at the versions this flake provides
      typstPins = lib.importTOML ./crates/csan-typst/tools.toml;

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
              # the Ruby gem, less its crate (ext/csan), a workspace member
              ./crates/csan-rb/Gemfile
              ./crates/csan-rb/Gemfile.lock
              ./crates/csan-rb/LICENSE-APACHE
              ./crates/csan-rb/LICENSE-MIT
              ./crates/csan-rb/README.md
              ./crates/csan-rb/Rakefile
              ./crates/csan-rb/csan.gemspec
              ./crates/csan-rb/exe
              ./crates/csan-rb/lib
              ./crates/csan-rb/sig
              ./crates/csan-rb/test
              ./crates/csan-typst/justfile
              ./crates/csan-typst/pkg
              ./crates/csan-typst/tools.toml
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
          # `nix fmt`: the Nix, the Typst (typstyle) and the Typst harness's
          # justfile (just's own, as `just --fmt` takes one file). The Rust
          # is rustfmt's (`just fmt`), the Markdown, TOML and JSON dprint's.
          formatter = pkgs.treefmt.withConfig {
            settings = {
              on-unmatched = "info";
              formatter = {
                nixfmt = {
                  command = "nixfmt";
                  includes = [ "*.nix" ];
                };
                typstyle = {
                  command = "typstyle";
                  options = [ "--inplace" ];
                  includes = [ "*.typ" ];
                };
                just = {
                  command = "just";
                  options = [
                    "--fmt"
                    "--unstable"
                    "--justfile"
                  ];
                  includes = [ "crates/csan-typst/justfile" ];
                };
              };
            };
            runtimeInputs = [
              pkgs.nixfmt
              pkgs.typstyle
              pkgs.just
            ];
          };
          # The Typst harness (crates/csan-typst/justfile): the tools
          # tools.toml pins, each the version this flake provides
          typst-package-check = package-check.packages.${pkgs.stdenv.hostPlatform.system}.default;
          typstTools = {
            inherit (pkgs)
              typst
              tytanic
              typstyle
              tinymist
              just
              shellcheck
              shfmt
              actionlint
              ;
            inherit typst-package-check;
          };
          stalePins = lib.filter (name: (typstPins.${name} or null) != typstTools.${name}.version) (
            lib.attrNames typstTools
          );
          unknownPins = lib.subtractLists (lib.attrNames typstTools) (lib.attrNames typstPins);
          # what the harness reads: its justfile, its pins and the package,
          # so an edit elsewhere reruns none of its checks (its
          # `versions-check` sees the package's imports alone here; Rust
          # CI's Typst job checks the whole repository's)
          typstSource = lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions [
              ./crates/csan-typst/justfile
              ./crates/csan-typst/tools.toml
              ./crates/csan-typst/pkg
            ];
          };
          # a harness check: RECIPES of crates/csan-typst/justfile on a copy
          # of typstSource, in a repository of its own with a commit (the
          # bundle is git's files). The sandbox has no /usr/bin/env, which
          # the script recipes name: their bash is named by its path instead.
          typstCheck =
            name: recipes:
            pkgs.runCommand "typst-${name}"
              {
                nativeBuildInputs = lib.attrValues typstTools ++ [
                  pkgs.jq
                  pkgs.gitMinimal
                ];
              }
              ''
                cp -r ${typstSource} "$TMPDIR/repository"
                chmod -R u+w "$TMPDIR/repository"
                cd "$TMPDIR/repository"
                export HOME="$TMPDIR"
                substituteInPlace crates/csan-typst/justfile \
                  --replace-fail '#!/usr/bin/env bash' '#!${pkgs.bash}/bin/bash'
                git init --quiet
                git add --all
                git -c user.name=check -c user.email=check@localhost commit --quiet -m check
                cd crates/csan-typst
                just ${recipes}
                touch $out
              '';
          common = {
            inherit version src;
            cargoLock.lockFile = ./Cargo.lock;
          };
          # a check: one cargo command over the whole workspace, offline
          # against the vendored lock, a stamp for output. The Python
          # bindings' build script (pyo3) needs an interpreter to configure
          # against; the Ruby gem's (rb-sys) a Ruby, and libclang for
          # bindgen.
          cargoCheck =
            name: command:
            rustPlatform.buildRustPackage (
              common
              // {
                pname = "${pname}-${name}";
                nativeBuildInputs = [
                  pkgs.python3
                  pkgs.ruby
                  rustPlatform.bindgenHook
                ];
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
                # the bindings: Python (maturin, pytest, and mypy for the
                # stubs' stubtest), Ruby (with bundler; libclang, through
                # bindgenHook, for rb-sys), npm (wasm-pack, node, binaryen's
                # wasm-opt)
                pkgs.maturin
                (pkgs.python3.withPackages (python: [
                  python.pytest
                  python.mypy
                ]))
                pkgs.ruby
                rustPlatform.bindgenHook
                pkgs.wasm-pack
                pkgs.binaryen
                pkgs.nodejs
              ]
              # the Typst package's harness (`just typst ...`): typst,
              # tytanic, typstyle, tinymist, typst-package-check, shellcheck,
              # shfmt, actionlint, as crates/csan-typst/tools.toml pins them
              ++ lib.attrValues typstTools;
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
            # every pin is the version this flake provides; after `nix
            # flake update`, the tools that moved, named for tools.toml
            typst-pins =
              pkgs.runCommand "typst-pins"
                {
                  report = lib.concatStringsSep "\n" (
                    map (
                      name:
                      "${name}: crates/csan-typst/tools.toml pins ${typstPins.${name} or "none"}, the flake has ${typstTools.${name}.version}"
                    ) stalePins
                    ++ map (
                      name: "${name}: pinned in crates/csan-typst/tools.toml, but no tool the flake provides"
                    ) unknownPins
                  );
                }
                ''
                  if [ -n "$report" ]; then
                    printf '%s\n' "$report" >&2
                    exit 1
                  fi
                  touch $out
                '';
            # built, not just evaluated: nixpkgs carries no build of it, so
            # a `nix flake update` its build no longer accepts fails here
            inherit typst-package-check;
            # the Typst package's tests (tytanic), its checks and its
            # justfile's shell (`just shell-check`)
            typst-tests = typstCheck "tests" "test";
            typst-package = typstCheck "package" "versions-check readme-check examples-check figures-check package-check lint";
            typst-shell = typstCheck "shell" "shell-check";
            # actionlint on a copy, in a repository of its own, as it runs
            # from the root: with no file named, it finds the workflows, and
            # .github/actionlint.yaml
            workflows =
              pkgs.runCommand "workflows"
                {
                  nativeBuildInputs = [
                    pkgs.actionlint
                    pkgs.shellcheck
                    pkgs.gitMinimal
                  ];
                }
                ''
                  cp -r ${
                    lib.fileset.toSource {
                      root = ./.;
                      fileset = ./.github;
                    }
                  } "$TMPDIR/repository"
                  chmod -R u+w "$TMPDIR/repository"
                  cd "$TMPDIR/repository"
                  export HOME="$TMPDIR"
                  git init --quiet
                  actionlint
                  touch $out
                '';
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
