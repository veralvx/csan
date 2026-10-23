# Contributing to `csan`

Thank you for considering contributing!

## Prerequisites

To build and test locally, you will need the following tools:

- **Rust:** Version 1.97 or later (stable; pinned in `rust-toolchain.toml`).
- **Just:** A command runner for our project automation.
- **Dprint:** Used for standardizing markdown and JSON formatting.
- **Cocogitto:** Used to enforce Conventional Commits.
- **cargo-audit and nightly Rust with Miri:** `just checks` also runs
  `cargo audit` and `just miri` (`rustup toolchain install nightly
  --component miri`).
- **For releasing:** git-cliff and jq, which `just release` uses.
- **Optional, for the Typst plugin:** the `wasm32-unknown-unknown` target
  (`rustup target add wasm32-unknown-unknown`), the `typst` CLI (0.14.2+)
  to run the plugin test documents, and `wasm-opt` (binaryen) if you want
  to shrink the packaged artifact.
- **Optional, for the bindings:** `maturin` for the Python package
  (`crates/csan-py`) and `wasm-pack` plus Node for the npm package
  (`crates/csan-web`).

With [Nix](https://nixos.org), `nix develop` provides all of the above, at
the versions `flake.lock` pins: the pinned toolchain (with the wasm target),
just, dprint, cocogitto, git-cliff, cargo-audit, maturin, wasm-pack, Node,
binaryen, and Typst. `nix develop .#miri` is a nightly shell for Miri, which
`just miri` uses on its own inside a Nix shell. `nix flake check` builds
what CI's `nix.yml` checks, and `nix fmt` formats the Nix files.

## Setup

1. Fork this repository and create your branch from `main`.
2. Clone your forked repository locally:

```sh
git clone https://github.com/<your-username>/csan && cd csan
```

## Architecture and Guidelines

The workspace centers on `crates/csan-core` (the cutter number library —
normalization, models, generators, and the Cutter-Sanborn table), wrapped
by four thin frontends: `crates/csan` (the `csan` binary),
`crates/csan-typst` (the Typst WASM plugin, with the Typst package under
`crates/csan-typst/pkg/`), `crates/csan-py` (Python bindings), and
`crates/csan-web` (JavaScript/npm bindings). Frontends stay thin: shared
input handling belongs in the library (`Attribution::from_parts`,
`CutterConfig::from_overrides`), not duplicated per binding. Lint policy
lives in the workspace `Cargo.toml` (`[workspace.lints]`) and
`clippy.toml`, and each allow there documents its reason. Please keep new
code warning-free under `just clippy` rather than adding blanket allows.

### Building the Typst plugin

The compiled plugin (`csan_typst.wasm`) is committed alongside the Typst
package, so Typst users don't need a Rust toolchain. To rebuild it after
changing the Rust code:

```sh
rustup target add wasm32-unknown-unknown
just wasm        # build + install into crates/csan-typst/pkg
just wasm-opt    # optional: shrink with binaryen's wasm-opt
just typst-test  # compile the assert-based plugin test suite
```

The npm package's build recipes are described in
[`crates/csan-web/README.md`](crates/csan-web/README.md#building-from-source),
the Python package's in
[`crates/csan-py/README.md`](crates/csan-py/README.md#building-from-source).

## Testing Strategy

When adding new features or fixing bugs, please keep the following in mind:

- **Pure Functions:** Isolate core logic from side effects wherever possible and cover it with pure unit tests.
- **Regressions:** If you are fixing a bug, include a test that explicitly reproduces the previous failure state.

## Development Workflow

1. We enforce the [Conventional Commits](https://www.conventionalcommits.org/) specification.
2. Run our automated checks locally. Our repository includes a `justfile` that mirrors our GitHub Actions CI pipeline.

```sh
just checks
```

This runs the core CI gates: `cargo check`/`test`/`clippy -D warnings`
(all targets, with the `cargo`, `pedantic`, and `restriction` clippy groups
enabled), `cargo fmt --check`, `cargo doc -D warnings`, `cargo audit`, Miri,
`dprint check`, and `cog check` for Conventional Commits. If `just checks`
passes on your machine, your code likely passes on CI, which additionally
builds and smoke-tests the Typst plugin, the Python wheel, and the npm
package.

## Releasing

`just release <major|minor|patch>` (on an up-to-date, clean `main`) bumps
the workspace version, the `csan-core` dependency pin, and the Typst
package version, prepends the release's section to `CHANGELOG.md` with
git-cliff, then commits, tags, and pushes. It refuses a bump smaller than
the unreleased breaking changes require. The tag starts the Release
workflow, which uses [dist](https://opensource.axo.dev/cargo-dist/) to
build the `csan` binaries and installers and attaches, to the same GitHub
release:

- the Typst plugin (`csan_typst.wasm`) and package
  (`csan_typst-<version>.tar.gz`);
- the Python wheels (abi3, Linux/musllinux/macOS/Windows on x86_64 and
  aarch64) and sdist;
- the Node npm package (`veralvx-csan-<version>.tgz`).

Three workflows then publish the release, each only once the Release
workflow has succeeded and each with trusted publishing (the one-time setup
is described at the top of each file):

- `publish-pypi.yml` uploads the release's wheels and sdist to
  [PyPI](https://pypi.org/project/csan/) (it runs when the Release
  workflow completes);
- `publish-npm.yml` publishes the release's npm tarball as
  [`@veralvx/csan`](https://www.npmjs.com/package/@veralvx/csan);
- `publish-crates.yml` publishes `csan-core`, then `csan`, from the tag to
  [crates.io](https://crates.io/crates/csan).

The npm and crates.io workflows start on the tag push and wait for that
tag's Release run, since crates.io trusted publishing does not accept
`workflow_run` events. Each workflow can also be run by hand for a release
tag, and skips whatever that release already published.

### Typst Universe

The Typst package is published by hand, with a pull request to
[typst/packages](https://github.com/typst/packages) after the release:

1. Copy `crates/csan-typst/pkg` from the release tag to
   `packages/preview/csan/<version>/` in a fork of typst/packages, without
   `tests/` (Typst Universe asks that test files not be submitted).
   `examples/` stays: the README links `examples/example.typ` and shows
   `examples/example.png`, and `typst.toml` already excludes it from the
   downloaded bundle.
2. `just release` has already set the version in `typst.toml` and in the
   README's `@preview/csan:<version>` import.
3. If the example changed, re-render its image first with
   `just typst-example`.

## Creating a Pull Request

1. Ensure your code passes `just checks` locally.
2. Open a Pull Request against the `main` branch.
3. In your PR description, clearly outline the problem you are solving. Link to the relevant open issue (e.g., `Fixes #123`), if any.
4. Wait for a maintainer to review your code.
