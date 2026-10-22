default:
  @just --list --unsorted


WASM_TARGET := "wasm32-unknown-unknown"
WASM_PROFILE := "wasm"
PLUGIN_CRATE := "csan-typst"
RAW_PLUGIN_WASM := "target" / WASM_TARGET / WASM_PROFILE / "csan_typst.wasm"
TYPST_PKG_DIR := "crates/csan-typst/pkg"
PLUGIN_WASM := TYPST_PKG_DIR / "csan_typst.wasm"
WEB_CRATE_DIR := "crates/csan-web"
# Where the `*-pack` recipes put release artifacts.
DIST_EXTRA_DIR := "target/dist-extra"

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

[doc("Package the Typst plugin and package for a release (requires `typst` for the tests)")]
[group("WASM")]
typst-pack out=DIST_EXTRA_DIR: wasm typst-test
  #!/usr/bin/env bash
  set -euo pipefail
  version="$(awk -F'"' '/^version[[:space:]]*=/ { print $2; exit }' {{ TYPST_PKG_DIR }}/typst.toml)"
  mkdir -p "{{ out }}"
  cp {{ PLUGIN_WASM }} "{{ out }}/csan_typst.wasm"
  # Flat archive of the package as Typst expects it under
  # {data-dir}/typst/packages/local/csan/<version>/ (the tests and examples
  # are excluded, as in typst.toml).
  tar -czf "{{ out }}/csan_typst-$version.tar.gz" -C {{ TYPST_PKG_DIR }} \
    --exclude=./tests --exclude=./examples --exclude='*.pdf' .
  echo "Packed {{ out }}/csan_typst.wasm and {{ out }}/csan_typst-$version.tar.gz"

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
  wasm-pack build {{ WEB_CRATE_DIR }} --release --target bundler

[doc("Build the Node npm package (requires wasm-pack)")]
[group("Bindings")]
npm-build-node:
  wasm-pack build {{ WEB_CRATE_DIR }} --release --target nodejs --out-dir pkg-node
  # wasm-pack copies the license files into the package but leaves them out
  # of package.json `files`, and npm only auto-includes `LICENSE[.ext]`.
  node -e ' \
    const fs = require("node:fs"); \
    const path = "{{ WEB_CRATE_DIR }}/pkg-node/package.json"; \
    const pkg = JSON.parse(fs.readFileSync(path, "utf8")); \
    pkg.files = [...new Set([...pkg.files, "LICENSE-APACHE", "LICENSE-MIT"])]; \
    fs.writeFileSync(path, JSON.stringify(pkg, null, 2) + "\n");'

[doc("Build the Node npm package and run its demo and smoke tests (requires wasm-pack and node)")]
[group("Bindings")]
npm-test: npm-build-node
  node {{ WEB_CRATE_DIR }}/examples/node-demo.cjs
  node {{ WEB_CRATE_DIR }}/examples/smoke-test.cjs

[doc("Pack the tested Node npm package into a tarball for a release (requires wasm-pack, node, npm)")]
[group("Bindings")]
npm-pack out=DIST_EXTRA_DIR: npm-test
  mkdir -p {{ out }}
  npm pack ./{{ WEB_CRATE_DIR }}/pkg-node --pack-destination {{ out }}

[doc("Run the web crate's wasm-bindgen tests in Node (requires wasm-pack and node)")]
[group("Bindings")]
web-test:
  wasm-pack test --node {{ WEB_CRATE_DIR }}

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

  # 3. Safety: local main must not be behind remote (tags are fetched too,
  #    so git-cliff bumps from the latest released version).
  git fetch --tags origin
  if [ "$(git rev-list --count HEAD..origin/main)" -ne 0 ]; then
    echo "Error: Your local 'main' branch is behind 'origin/main'. Please pull first." >&2
    exit 1
  fi

  # 4. Compute the bumped version tag
  # Handles both git-cliff v1 (array) and v2+ (object with .releases) JSON.
  # git-cliff 2.x reports a bumped version even with nothing to release, so
  # the unreleased commits are counted as well.
  release_json="$(git cliff --unreleased --bump {{ semver }} --context \
    | jq -c 'if type == "array" then .[0] else .releases[0] end')"
  tag="$(jq -r '.version' <<<"$release_json")"
  unreleased="$(jq -r '.commits | length' <<<"$release_json")"

  if [ "$unreleased" -eq 0 ]; then
    echo "Error: no unreleased commits since the latest tag; nothing to release." >&2
    exit 1
  fi
  if [ -z "$tag" ] || [ "$tag" = "null" ]; then
    echo "Error: Could not determine bumped version. Are there unreleased conventional commits?" >&2
    exit 1
  fi

  # Strip leading 'v' for Cargo (e.g. v1.2.3 → 1.2.3)
  raw_version="${tag#v}"
  if ! [[ "$raw_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Error: unexpected version '$tag' from git-cliff." >&2
    exit 1
  fi
  if git rev-parse -q --verify "refs/tags/$tag" >/dev/null; then
    echo "Error: tag $tag already exists." >&2
    exit 1
  fi

  # If anything below fails before the commit, restore the modified files.
  # Uses HEAD (not index) so cleanup works even after git-add.
  release_files=(Cargo.toml Cargo.lock CHANGELOG.md crates/csan-typst/pkg/typst.toml)
  trap 'echo "Release aborted — restoring modified files." >&2; git checkout HEAD -- "${release_files[@]}" 2>/dev/null' ERR

  # 5. Bump the version. Every crate inherits `[workspace.package] version`;
  #    the `csan-core` workspace dependency pins the same version (needed to
  #    publish `csan` against it), and the Typst package manifest follows.
  #    Each edit must hit exactly the expected lines, or the release stops.
  awk -v ver="$raw_version" '
    /^\[/ { section = $0 }
    section == "[workspace.package]" && /^version[[:space:]]*=/ {
      print "version = \"" ver "\""; hits++; next
    }
    section == "[workspace.dependencies]" && /^csan-core[[:space:]]*=/ {
      hits += sub(/version = "[^"]*"/, "version = \"" ver "\"")
    }
    { print }
    END { if (hits != 2) { print "Error: expected 2 version fields in Cargo.toml, updated " hits+0 > "/dev/stderr"; exit 1 } }
  ' Cargo.toml > Cargo.toml.tmp && mv Cargo.toml.tmp Cargo.toml
  awk -v ver="$raw_version" '
    /^\[/ { section = $0 }
    section == "[package]" && /^version[[:space:]]*=/ && !hits {
      print "version = \"" ver "\""; hits++; next
    }
    { print }
    END { if (hits != 1) { print "Error: no [package] version in typst.toml" > "/dev/stderr"; exit 1 } }
  ' crates/csan-typst/pkg/typst.toml > typst.toml.tmp && mv typst.toml.tmp crates/csan-typst/pkg/typst.toml

  # 6. Refresh the workspace members' versions in Cargo.lock
  cargo update --workspace

  # 7. Prepend this release's section (for the tag computed above) to the
  #    changelog. Earlier sections, including hand edits, are kept as is,
  #    and a shallow clone cannot drop the history it lacks.
  git cliff --unreleased --tag "$tag" --prepend CHANGELOG.md

  # 8. Stage exactly the files modified by the release process
  git add "${release_files[@]}"

  # 9. Commit and tag — clear the trap first: once committed, a push
  #    failure is solved by retrying the push, not by reverting files.
  trap - ERR
  git commit -m "chore(release): $tag"
  git tag -a "$tag" -m "Release $tag"

  # 10. Push branch and tag atomically (both or neither). The tag push
  #     triggers the Release workflow, and a successful release triggers
  #     the PyPI publish.
  git push --atomic origin main "$tag"

  echo "Released $tag"


clean:
  @echo "Cleaning build artifacts..."
  cargo clean
  @echo "Done."
