# csan

[![crates.io](https://img.shields.io/crates/v/csan.svg)](https://crates.io/crates/csan)
[![PyPI](https://img.shields.io/pypi/v/csan.svg)](https://pypi.org/project/csan/)
[![npm](https://img.shields.io/npm/v/@veralvx/csan.svg)](https://www.npmjs.com/package/@veralvx/csan)
[![CI](https://github.com/veralvx/csan/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/veralvx/csan/actions/workflows/rust-ci.yml)

`csan` generates Cutter-Sanborn and Library of Congress cutter numbers for
library cataloging from author names and titles. It is a command-line tool
and a library for Rust, Python, JavaScript (npm), and Typst.

```console
$ csan --forename Charles --surname Dickens --title "Oliver Twist"
D548o
$ csan --method cutter-loc --forename Charles --surname Dickens
D535
```

It implements two methods:

- **Cutter-Sanborn three-figure:** a lookup against the bundled
  Cutter-Sanborn table (12,000+ entries), with an optional call number
  prefix and a lowercase work mark derived from the title.
- **Library of Congress:** the LoC character-branching rules
  ([Cutter table](https://www.loc.gov/aba/pcc/053/table.html),
  [CSM G 63](https://www.loc.gov/aba/publications/FreeCSM/G063.pdf)),
  producing an initial plus a digit sequence.

Input names are normalized first: whitespace and punctuation are stripped,
diacritics and ligatures are folded to ASCII (`Čapek` → `Capek`, `Strauß` →
`Strauss`), `Mc`/`M'` prefixes expand to `Mac`, and a surname of
`St`/`Ste` (as in Saint Patrick) expands to `Saint`/`Sainte`, matching the
conventions of the underlying tables.
Anonymous works are cuttered by title.

## Install

- **Command-line tool:** `cargo install csan`, or prebuilt with
  `cargo binstall csan` or the installers and archives on each
  [GitHub release](https://github.com/veralvx/csan/releases). With Nix
  (Linux, aarch64 macOS): `nix run github:veralvx/csan -- --help`. Without
  installing: `npx @veralvx/csan --help`.
- **Rust library:** `cargo add csan-core`.
- **Python:** `pip install csan` (library and CLI), or
  `uv tool install csan` for the CLI alone.
- **JavaScript (Node):** `npm install @veralvx/csan` (library and CLI).
- **Typst:** see [Typst](#typst).

## Command line

```sh
csan --surname Dickens --title "Oliver Twist"            # D548o
csan --method cutter-loc --surname Carter                # C378
csan --method cutter-loc --title "100 best recipes"      # A153
csan --surname Borges --json                             # {"cutter_sanborn": {"formatted": "B732", ...}}
```

`--method` selects the method, `cutter-sanborn` (the default) or
`cutter-loc`, and at least one non-blank input (`--forename`, `--surname`,
`--title`) is required. `--prefix-len`/`--suffix-len` tune the
Cutter-Sanborn output, `--max-len` the LoC digits, and `--json` emits the
parts as a JSON object keyed by method (`cutter_sanborn` or
`cutter_loc`). Errors exit non-zero with a message on stderr.

With [Nix](https://nixos.org), the flake builds the binary with the
toolchain that `rust-toolchain.toml` pins:

```sh
nix run github:veralvx/csan -- --help
nix build github:veralvx/csan     # ./result/bin/csan
nix profile install github:veralvx/csan
```

## Rust

The library is the [`csan-core`](https://crates.io/crates/csan-core) crate
(up to 0.5.x it was published as `csan`, a name that now belongs to the
command-line tool):

```rust
use csan_core::{
    Attribution, Author, CutterConfig, CutterGenerator, CutterLocMethod,
    CutterSanbornMethod, Opus,
};

fn main() -> Result<(), csan_core::NormalizationError> {
    let entry = Attribution::Both {
        author: Author::new("Charles", "Dickens"),
        opus: Opus::new("Oliver Twist"),
    };

    let sanborn = CutterSanbornMethod::generate(&entry, CutterConfig::default())?;
    assert_eq!(sanborn.to_string(), "D548o");

    let loc = CutterLocMethod::generate(&entry, CutterConfig::default())?;
    assert_eq!(loc.to_string(), "D535");
    Ok(())
}
```

`CutterConfig` tunes the output: `prefix_len` and `suffix_len` control the
Cutter-Sanborn prefix and work mark, and `max_len` controls the LoC digit
count (clamped to 4). Works without an author use
`Attribution::OpusOnly(...)` and are cuttered by title, including the LoC
numeral rule (`"100 best recipes"` → `A153`). The API reference is on
[docs.rs](https://docs.rs/csan-core).

## Python

The PyPI package [`csan`](https://pypi.org/project/csan/) (CPython 3.9+,
abi3 wheels) provides the library and the `csan` command:

```python
import csan

csan.cutter_sanborn(surname="Dickens", title="Oliver Twist")  # 'D548o'
csan.cutter_loc_detailed(surname="Carter")
# {'alpha_prefix': 'C', 'formatted': 'C378', 'numeric': 378}
```

See the [Python package README](https://github.com/veralvx/csan/blob/main/crates/csan-py/README.md)
for the full API.

## JavaScript

The npm package [`@veralvx/csan`](https://www.npmjs.com/package/@veralvx/csan)
is the library compiled to WebAssembly, built for Node (CommonJS, with
TypeScript declarations):

```js
const { cutterLoc, cutterSanborn } = require("@veralvx/csan");

cutterSanborn({ surname: "Dickens", title: "Oliver Twist" }); // "D548o"
cutterLoc({ surname: "Carter" }); // "C378"
```

It also provides the `csan` command, with the same interface as the native
binary. See the [npm package README](https://github.com/veralvx/csan/blob/main/crates/csan-web/README.md)
for the full API and the browser build.

## Typst

The Typst package exposes two functions:

```typ
// Replace <version> with the installed release.
#import "@local/csan:<version>": cutter-sanborn, cutter-loc

#cutter-sanborn(
  forename: "Charles",
  surname: "Dickens",
  title: "Oliver Twist",
) // -> "D548o"

#cutter-loc(surname: "Carter") // -> "C378"
```

Both accept `forename`, `surname`, and `title` (at least one must be
non-blank; blank strings count as absent). `cutter-sanborn` also takes
`prefix-len` and `suffix-len`; `cutter-loc` takes `max-len`. Pass
`detailed: true` to receive a dictionary of the parts (`alpha_prefix`,
`numeric`, …) instead of the formatted string. Requires Typst 0.14.2 or
later.

Until the package is published to Typst Universe, install it as a
[local package](https://github.com/typst/packages#local-packages): extract
the `csan_typst-<version>.tar.gz` attached to each
[GitHub release](https://github.com/veralvx/csan/releases) into
`{data-dir}/typst/packages/local/csan/<version>/`, then import
`@local/csan:<version>`. The plugin binary alone is attached as
`csan_typst.wasm`.

## Expected values

Generated by `cargo run --example showcase` with the default configuration:

| Author              | Cutter-Sanborn | Library of Congress |
| ------------------- | -------------- | ------------------- |
| Jane Austen         | `A933`         | `A978`              |
| Eric Blair          | `B635`         | `B535`              |
| Agatha Christie     | `C555`         | `C475`              |
| Samuel Clemens      | `C625`         | `C546`              |
| Jorge De la Cruz    | `D332`         | `D453`              |
| Charles Dickens     | `D548`         | `D535`              |
| Emily Dickinson     | `D553`         | `D535`              |
| Fyodor Dostoyevsky  | `D724`         | `D678`              |
| Stephen King        | `K54`          | `K564`              |
| VERA LVX            | `L979`         | `L89`               |
| Herman Melville     | `M531`         | `M458`              |
| George Orwell       | `O79`          | `O794`              |
| William Shakespeare | `S527`         | `S535`              |
| Lord Sith           | `S623`         | `S584`              |
| Ivan Smith          | `S649`         | `S658`              |
| William Smith       | `S664`         | `S658`              |
| Leo Tolstoy         | `T654`         | `T657`              |
| Mark Twain          | `T969`         | `T835`              |
| Virginia Woolf      | `W913`         | `W665`              |
| Emile Zola          | `Z86`          | `Z653`              |

## Contributing

| Path                    | Contents                                                    |
| ----------------------- | ----------------------------------------------------------- |
| `crates/csan-core`      | The library: normalization, models, generators, the table.  |
| `crates/csan`           | The `csan` command-line tool (clap).                        |
| `crates/csan-py`        | Python bindings and the PyPI package (pyo3 + maturin).      |
| `crates/csan-web`       | JavaScript/npm bindings via WebAssembly (wasm-bindgen).     |
| `crates/csan-typst`     | The Typst WASM plugin wrapping the library.                 |
| `crates/csan-typst/pkg` | The Typst package (`typst.toml`, `lib.typ`, plugin binary). |

[CONTRIBUTING.md](https://github.com/veralvx/csan/blob/main/CONTRIBUTING.md)
covers the development setup, the checks CI runs, building the Typst
plugin, and the release process. Changes are listed in the
[changelog](https://github.com/veralvx/csan/blob/main/CHANGELOG.md).

## License

Licensed under either of
[Apache License, Version 2.0](https://github.com/veralvx/csan/blob/main/LICENSE-APACHE)
or [MIT license](https://github.com/veralvx/csan/blob/main/LICENSE-MIT)
at your option.
