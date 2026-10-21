default:
  @just --list --unsorted


WASM_TARGET := "wasm32-unknown-unknown"
WASM_PROFILE := "wasm"
PLUGIN_CRATE := "csan-typst"
RAW_PLUGIN_WASM := "target" / WASM_TARGET / WASM_PROFILE / "csan_typst.wasm"
TYPST_PKG_DIR := "crates/csan-typst/pkg"
PLUGIN_WASM := TYPST_PKG_DIR / "csan_typst.wasm"

# ------------------------------------------------------------------ WASM

[doc("Compile the Typst plugin (requires `rustup target add wasm32-unknown-unknown`)")]
[group("WASM")]
wasm-build:
  cargo build --target {{ WASM_TARGET }} --profile {{ WASM_PROFILE }} -p {{ PLUGIN_CRATE }}

[doc("Build the plugin and install it into the Typst package")]
[group("WASM")]
wasm: wasm-build
  cp {{ RAW_PLUGIN_WASM }} {{ PLUGIN_WASM }}

[doc("Optional: shrink the packaged wasm with wasm-opt (binaryen)")]
[group("WASM")]
[group("Optimization")]
wasm-opt: wasm
  # --all-features is deliberately avoided: older binaryen re-encodes with
  # GC heap types, which Typst's wasmi runtime rejects. Only the features
  # rustc actually emits for this target are enabled.
  wasm-opt -Oz \
    --enable-bulk-memory --enable-sign-ext --enable-mutable-globals \
    --enable-nontrapping-float-to-int \
    --strip-debug --strip-producers \
    {{ PLUGIN_WASM }} -o {{ PLUGIN_WASM }}

[doc("Compile the Typst package's assert-based test document (requires `typst`)")]
[group("WASM")]
typst-test:
  typst compile --root {{ TYPST_PKG_DIR }} {{ TYPST_PKG_DIR }}/tests/test.typ /tmp/csan-test.pdf
  typst compile --root {{ TYPST_PKG_DIR }} {{ TYPST_PKG_DIR }}/examples/example.typ /tmp/csan-example.pdf
  @echo "Typst plugin test suite passed."

# ---------------------------------------------------------------- Bindings

[doc("Build and install the Python package into the active environment (requires maturin)")]
[group("Bindings")]
py-develop:
  maturin develop --release -m crates/csan-py/Cargo.toml

[doc("Build a release wheel for the Python package (requires maturin)")]
[group("Bindings")]
py-build:
  maturin build --release -m crates/csan-py/Cargo.toml

[doc("Build the npm package with wasm-pack (bundler target, into crates/csan-web/pkg)")]
[group("Bindings")]
npm-build:
  wasm-pack build crates/csan-web --release --target bundler

[doc("Build the Node npm package and run its demo (requires wasm-pack and node)")]
[group("Bindings")]
npm-test:
  wasm-pack build crates/csan-web --release --target nodejs --out-dir pkg-node
  node crates/csan-web/examples/node-demo.cjs

[doc("Run the web crate's wasm-bindgen tests in Node (requires wasm-pack and node)")]
[group("Bindings")]
web-test:
  wasm-pack test --node crates/csan-web

[doc("Build the Python module into the active venv and run its pytest suite (requires maturin and pytest)")]
[group("Bindings")]
py-test:
  maturin develop -m crates/csan-py/Cargo.toml
  pytest crates/csan-py/tests

# ----------------------------------------------------------------- Checks

cargo-check:
  cargo check --workspace --all-targets --all-features
  cargo check --workspace --all-targets --no-default-features

test:
  cargo test --workspace --all-features
  cargo test --workspace --no-default-features

clippy:
  #!/bin/sh
  export RUSTFLAGS="-Dwarnings"
  cargo clippy --all-targets --all-features --workspace
  cargo clippy --all-targets --no-default-features --workspace

miri:
  cargo +nightly miri test

fmt-check:
  cargo fmt --check

docs-check:
  #!/bin/sh
  export RUSTDOCFLAGS="-Dwarnings"
  cargo doc --workspace --no-deps --all-features

cog:
  cog check

dprint-check:
  dprint check

audit:
  cargo audit

checks:
  just cargo-check
  just test
  just miri
  just clippy
  just fmt-check
  just docs-check
  just dprint-check
  just cog
  just audit

fmt:
  cargo fmt
  dprint fmt

# ---------------------------------------------------------------- Release

[doc("Release a new version: [major|minor|patch]")]
release semver:
  #!/usr/bin/env bash
  set -euo pipefail

  # 0. Validate argument
  case "{{ semver }}" in
    major|minor|patch) ;;
    *) echo "Error: argument must be 'major', 'minor', or 'patch' (got '{{ semver }}')" >&2; exit 1 ;;
  esac

  # 1. Safety: working directory must be clean
  if [ -n "$(git status --porcelain)" ]; then
    echo "Error: Working directory is not clean. Commit or stash your changes first." >&2
    exit 1
  fi

  # 2. Safety: must be on main
  current_branch="$(git branch --show-current)"
  if [ "$current_branch" != "main" ]; then
    echo "Error: You must be on the 'main' branch to cut a release (on '$current_branch')." >&2
    exit 1
  fi

  # 3. Safety: local main must not be behind remote
  git fetch origin
  if [ "$(git rev-list --count HEAD..origin/main)" -ne 0 ]; then
    echo "Error: Your local 'main' branch is behind 'origin/main'. Please pull first." >&2
    exit 1
  fi

  # 4. Compute the bumped version tag
  # Handles both git-cliff v1 (array) and v2+ (object with .releases) JSON.
  tag="$(git cliff --unreleased --bump {{ semver }} --context \
    | jq -r 'if type == "array" then .[0].version else .releases[0].version end')"

  if [ -z "$tag" ] || [ "$tag" = "null" ]; then
    echo "Error: Could not determine bumped version. Are there unreleased conventional commits?" >&2
    exit 1
  fi

  # If anything below fails before the commit, restore the modified files.
  # Uses HEAD (not index) so cleanup works even after git-add.
  trap 'echo "Release aborted — restoring modified files." >&2; git checkout HEAD -- crates/*/Cargo.toml crates/csan-typst/pkg/typst.toml Cargo.lock CHANGELOG.md 2>/dev/null' ERR

  # Strip leading 'v' for Cargo (e.g. v1.2.3 → 1.2.3)
  raw_version="${tag#v}"

  # 5. Bump the version in every workspace member (the workspace root
  #    carries no version) and keep the Typst package manifest in sync.
  #    Only the first top-level `version =` of each file is touched.
  for manifest in crates/*/Cargo.toml crates/csan-typst/pkg/typst.toml; do
    awk -v ver="$raw_version" '
      /^version[[:space:]]*=/ && !done {
        print "version = \"" ver "\""
        done = 1
        next
      }
      1
    ' "$manifest" > "$manifest.tmp" && mv "$manifest.tmp" "$manifest"
  done

  # 6. Regenerate Cargo.lock to reflect the new versions
  cargo check --quiet

  # 7. Regenerate the full changelog
  git cliff --bump {{ semver }} --output CHANGELOG.md

  # 8. Stage exactly the files modified by the release process
  git add crates/*/Cargo.toml crates/csan-typst/pkg/typst.toml Cargo.lock CHANGELOG.md

  # 9. Commit and tag — clear the trap first: once committed, a push
  #    failure is solved by retrying the push, not by reverting files.
  trap - ERR
  git commit -m "chore(release): $tag"
  git tag -a "$tag" -m "Release $tag"

  # 10. Push branch and tag atomically
  git push --follow-tags origin main

  echo "Released $tag"


clean:
  @echo "Cleaning build artifacts..."
  cargo clean
  @echo "Done."
