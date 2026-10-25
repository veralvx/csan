# Contributing to `csan`

Thank you for considering contributing!

## Prerequisites

To build and test locally, you will need the following tools:

- **Rust:** Version 1.97 or later (stable; pinned in `rust-toolchain.toml`).
- **Ruby and libclang:** Ruby 3.3+ with its headers, and libclang, since
  the Ruby bindings' crate is a workspace member (rb-sys reads the Ruby
  installation and generates its bindings with bindgen when it builds);
  plus Bundler for the gem's own recipes (`just rb-test`, …).
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
- **Optional, for the Typst package's harness** (`just typst ...`): the
  tools [`crates/csan-typst/tools.toml`](crates/csan-typst/tools.toml)
  pins, at those versions: typst, tytanic (`tt`), typstyle, tinymist,
  typst-package-check, just, shellcheck, shfmt and actionlint, plus jq.
- **Optional, for the bindings:** `maturin`, `pytest` and `mypy` (its
  stubtest checks the type stubs) for the Python package (`crates/csan-py`)
  and `wasm-pack` plus Node for the npm package
  (`crates/csan-web`). The gem (`crates/csan-rb`) needs only Ruby and
  Bundler, above; its cross-compiled platform gems are built in CI.

With [Nix](https://nixos.org), `nix develop` provides all of the above, at
the versions `flake.lock` pins: the pinned toolchain (with the wasm target),
Ruby with Bundler, libclang, just, dprint, cocogitto, git-cliff,
cargo-audit, maturin, wasm-pack, Node, binaryen, and the Typst harness's
tools. `nix develop .#miri` is a nightly
shell for Miri, which `just miri` uses on its own inside a Nix shell. `nix
flake check` builds what CI's `nix.yml` checks, and `nix fmt` formats the
Nix files, the Typst code (typstyle) and the Typst harness's justfile.

## Setup

1. Fork this repository and create your branch from `main`.
2. Clone your forked repository locally:

```sh
git clone https://github.com/<your-username>/csan && cd csan
```

## Architecture and Guidelines

The workspace centers on `crates/csan-core` (the cutter number library —
normalization, models, generators, and the Cutter-Sanborn table), wrapped
by five thin frontends: `crates/csan` (the `csan` binary),
`crates/csan-typst` (the Typst WASM plugin, with the Typst package under
`crates/csan-typst/pkg/`), `crates/csan-py` (Python bindings),
`crates/csan-rb` (Ruby bindings, the crate in `ext/csan/`), and
`crates/csan-web` (JavaScript/npm bindings). Frontends stay thin: shared
input handling belongs in the library (`Attribution::from_parts`,
`CutterConfig::from_overrides`, `Language::from_optional`,
`Language::for_rule`), not duplicated per binding. The languages and their
articles live in one table, the `languages!` macro in
`crates/csan-core/src/catalog/language.rs`, from which the rustdoc and
`csan --help` are generated. The README's "Initial articles" table and the
Typst package README's list copy it, so update them with it (`csan --help`
lists the names and codes). Lint policy
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

### The Typst package's harness

[`crates/csan-typst/justfile`](crates/csan-typst/justfile) holds the Typst
package's own recipes, run from the repository's root as `just typst
<recipe>` (or as `just <recipe>` in `crates/csan-typst`). They run in
`crates/csan-typst/pkg`, against the plugin committed there (rebuild it with
`just wasm` first), and `just typst` lists them:

```sh
just typst install         # the package into Typst's package path: #import "@local/csan:<version>"
just typst test            # the tests under pkg/tests/ (tytanic)
just typst figures         # re-render examples/example.png, which the README shows
just typst checks          # every check below
```

The checks: the pinned tools are the ones on PATH (`tools-check`), the
Typst code's and the justfile's format (`format-check`, typstyle),
tinymist's lint (`lint`), the justfile's script recipes (`shell-check`,
shellcheck and shfmt), the tests (`test`), every `@preview/csan` import at
`typst.toml`'s version (`versions-check`), the README's `typ` blocks and the
examples compiled against the bundle (`readme-check`, `examples-check`;
`floor` does both on the oldest supported Typst, as `typst-0.14.2`), the
README's figure as its example renders (`figures-check`), and
typst-package-check on the bundle (`package-check`, offline unless `just
typst::online=true typst package-check`). `just typst bundle` writes the package as
typst/packages gets it to `pkg/dist/preview/csan/<version>/`: `git
archive` of `pkg/`, less what `pkg/.gitattributes` export-ignores (the
tests).

`nix flake check` runs them in the sandbox (`typst-tests`, `typst-package`,
`typst-shell`), with `typst-pins`, which fails when a `nix flake update`
moved a tool from its `tools.toml` pin, naming it: update `tools.toml` to
the version the flake has. Rust CI's Typst job runs `versions-check`,
`readme-check`, `examples-check` and `floor` against the plugin it builds.

The npm package's build recipes are described in
[`crates/csan-web/README.md`](crates/csan-web/README.md#building-from-source),
the Python package's in
[`crates/csan-py/README.md`](crates/csan-py/README.md#building-from-source),
and the gem's in
[`crates/csan-rb/README.md`](crates/csan-rb/README.md#building-from-source)
(`just rb-test`, `just rb-lint` and `just rb-build` run them from the
root).

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
builds and smoke-tests the Typst plugin, the Python wheel, the gem (with
its tests and standardrb, on Ruby 3.3 and 4.0), and the npm package.

## Releasing

`just release <major|minor|patch>` (on an up-to-date, clean `main`) bumps
the workspace version, the `csan-core` dependency pin, the Typst
package version, and the gem's version (`lib/csan/version.rb`, its crate's
`csan` pin and `Gemfile.lock`), prepends the release's section to `CHANGELOG.md` with
git-cliff, then commits, tags, and pushes. It refuses a bump smaller than
the unreleased breaking changes require. The tag starts the Release
workflow, which uses [dist](https://opensource.axo.dev/cargo-dist/) to
build the `csan` binaries and installers and attaches, to the same GitHub
release:

- the Typst plugin (`csan_typst.wasm`) and package
  (`csan_typst-<version>.tar.gz`);
- the Python wheels (abi3, Linux/musllinux/macOS/Windows on x86_64 and
  aarch64) and sdist;
- the Ruby gems: one per platform (`csan-<version>-<platform>.gem`, for
  Ruby 3.3, 3.4 and 4.0 on Linux glibc/musl, macOS and Windows, x86_64 and
  arm64; Windows on arm64 from Ruby 3.4) and the source gem
  (`csan-<version>.gem`);
- the Node npm package (`veralvx-csan-<version>.tgz`).

Four workflows then publish the release, each only once the Release
workflow has succeeded and each with trusted publishing (the one-time setup
is described at the top of each file):

- `publish-pypi.yml` uploads the release's wheels and sdist to
  [PyPI](https://pypi.org/project/csan/) (it runs when the Release
  workflow completes);
- `publish-rubygems.yml` pushes the release's gems to
  [RubyGems](https://rubygems.org/gems/csan) (it runs when the Release
  workflow completes; the source gem, which builds against `csan` from
  crates.io, once crates.io has it);
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

1. On the release tag, run `just typst checks`, then copy what `just typst
   bundle` writes (`crates/csan-typst/pkg/dist/preview/csan/<version>/`)
   to `packages/preview/csan/<version>/` in a fork of typst/packages. It is
   `pkg/` without its tests, which Typst Universe asks not to be submitted;
   `examples/` stays, as the README links `examples/example.typ` and shows
   `examples/example.png`, and `typst.toml` excludes it from the download.
2. `just release` has already set the version in `typst.toml` and in every
   `@preview/csan:<version>` import (`just typst versions-check` holds them
   together).
3. If the example changed, re-render its image first with `just typst
   figures` (`just typst figures-check` fails until then).

## Creating a Pull Request

1. Ensure your code passes `just checks` locally.
2. Open a Pull Request against the `main` branch.
3. In your PR description, clearly outline the problem you are solving. Link to the relevant open issue (e.g., `Fixes #123`), if any.
4. Wait for a maintainer to review your code.
