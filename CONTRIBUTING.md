# Contributing to `csan`

Thank you for considering contributing!

## Prerequisites

To build and test locally, you will need the following tools:

- **Rust:** Version 1.97 or later (stable; pinned in `rust-toolchain.toml`).
- **Just:** A command runner for our project automation.
- **Dprint:** Used for standardizing markdown and JSON formatting.
- **Cocogitto:** Used to enforce Conventional Commits.
- **Optional, for the Typst plugin:** the `wasm32-unknown-unknown` target
  (`rustup target add wasm32-unknown-unknown`), the `typst` CLI (0.14.2+)
  to run the plugin test documents, and `wasm-opt` (binaryen) if you want
  to shrink the packaged artifact.
- **Optional, for the bindings:** `maturin` for the Python package
  (`crates/csan-py`) and `wasm-pack` plus Node for the npm package
  (`crates/csan-web`).

## Setup

1. Fork this repository and create your branch from `main`.
2. Clone your forked repository locally:

```sh
git clone https://github.com/veralvx/csan && cd csan
```

## Architecture and Guidelines

The workspace centers on `crates/csan-core` (the cutter number library —
normalization, models, generators, and the Cutter-Sanborn table), wrapped
by four thin frontends: `crates/csan` (the `csan` binary),
`crates/csan-typst` (the Typst WASM plugin, with the Typst package under
`crates/csan-typst/pkg/`), `crates/csan-py` (Python bindings), and
`crates/csan-web` (JavaScript/npm bindings). Frontends stay thin: shared
input handling belongs in the library (`Attribution::from_parts`,
`CutterConfig::from_overrides`), not duplicated per binding. The `README.md` covers the
layout and the wasm build flow; lint policy lives in the workspace
`Cargo.toml` (`[workspace.lints]`) and `clippy.toml`, and each allow there
documents its reason. Please keep new code warning-free under
`just clippy` rather than adding blanket allows.

## Testing Strategy

When adding new features or fixing bugs, please keep the following in mind:

- **Pure Functions:** Isolate core logic from side effects wherever possible and cover it with pure unit tests.
- **Regressions:** If you are fixing a bug, include a test that explicitly reproduces the previous failure state.

## Development Workflow

1. We enforce the [Conventional Commits](https://www.conventionalcommits.org/) specification.
2. Run our automated checks locally. Our repository includes a `Justfile` that mirrors our GitHub Actions CI pipeline.

```sh
just checks
```

This single command will run formatting (`cargo fmt`, `dprint`), linting (`cargo clippy -D warnings`), testing (`cargo test`), and commit validation (`cog check`). If `just checks` passes on your machine, your code likely passes on CI.

## Creating a Pull Request

1. Ensure your code passes `just checks` locally.
2. Open a Pull Request against the `main` branch.
3. In your PR description, clearly outline the problem you are solving. Link to the relevant open issue (e.g., `Fixes #123`), if any.
4. Wait for a maintainer to review your code.
