# csan (Python package)

Generate Cutter-Sanborn three-figure and Library of Congress cutter numbers
for library cataloging from author names and titles. The package provides
Python bindings to the Rust [`csan-core`](https://crates.io/crates/csan-core)
library and the `csan` command-line tool, as abi3 wheels for CPython 3.9+
on Linux, macOS, and Windows.

```sh
pip install csan        # library + CLI
uv tool install csan    # just the CLI, isolated
```

```python
import csan

csan.cutter_sanborn(forename="Charles", surname="Dickens",
                    title="Oliver Twist")  # 'D548o'
csan.cutter_loc(surname="Carter")  # 'C378'

csan.cutter_sanborn_detailed(surname="Borges")
# {'alpha_prefix': 'B', 'alpha_suffix': None, 'entry': 'Borg',
#  'formatted': 'B732', 'numeric': 732}
```

Every function accepts `forename`, `surname`, and `title` keyword
arguments (at least one must be non-blank; blank strings count as absent).
`cutter_sanborn*` also take `prefix_len` and `suffix_len`; `cutter_loc*`
take `max_len` (clamped to 4). The `*_detailed` variants return a dict of
the number's parts plus the rendered call number under `"formatted"`.

Every function also takes the keyword-only `lang` and `strip_article`:
with `strip_article=True`, the title is filed without its initial article
in the language `lang` names (an ISO 639 code, a BCP 47 tag or a POSIX
locale, case-insensitive; `csan --help` lists them all). `lang` alone
changes nothing.

```python
csan.cutter_sanborn(surname="Dickens", title="A Christmas Carol",
                    lang="en", strip_article=True)  # 'D548c'
csan.cutter_loc(title="The Bear", lang="en", strip_article=True)  # 'B437'
```

A blank `lang` counts as absent, and author names are never stripped.
Unusable input, an unknown `lang` (even with `strip_article` off), or
`strip_article` without `lang` raises `ValueError`, and a `lang` that is
not a string, or a `strip_article` that is not a bool, `TypeError`.

The console script is the same interface as the standalone Rust binary:

```sh
csan --forename Charles --surname Dickens --title "Oliver Twist"   # D548o
csan --method cutter-loc --surname Carter                         # C378
```

## Building from source

Built with [maturin](https://www.maturin.rs/) from
[`crates/csan-py`](https://github.com/veralvx/csan/tree/main/crates/csan-py)
in the main repository (Rust toolchain required):

```sh
pip install maturin
maturin develop -m crates/csan-py/Cargo.toml       # editable install
maturin build --release -m crates/csan-py/Cargo.toml
```

Each [GitHub release](https://github.com/veralvx/csan/releases) carries
the wheels (Linux manylinux and musllinux, macOS, and Windows, on x86_64
and aarch64) and the sdist that are published to PyPI.

## License

Licensed under either of
[Apache License, Version 2.0](https://github.com/veralvx/csan/blob/main/LICENSE-APACHE)
or [MIT license](https://github.com/veralvx/csan/blob/main/LICENSE-MIT)
at your option.
