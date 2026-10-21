# csan (Python package)

Cutter call numbers for library cataloging — Cutter-Sanborn three-figure
and Library of Congress — as fast Rust-powered Python bindings, including
the `csan` command-line tool.

```sh
pip install csan        # library + CLI
uv tool install csan    # just the CLI, isolated
```

```python
import csan

csan.cutter_sanborn(given_name="Charles", family_name="Dickens",
                      title="Oliver Twist")   # 'D548o'
csan.cutter_loc(family_name="Carter")       # 'C378'

csan.cutter_sanborn_detailed(family_name="Borges")
# {'alpha_prefix': 'B', 'numeric': 732, 'alpha_suffix': None,
#  'entry': 'Borg', 'formatted': 'B732'}
```

Every function accepts `given_name`, `family_name`, and `title` keyword
arguments (at least one must be non-blank; blank strings count as absent).
`cutter_sanborn*` also take `prefix_len` and `suffix_len`; `cutter_loc*`
take `max_len` (clamped to 4). The `*_detailed` variants return a dict of
the number's parts plus the rendered call number under `"formatted"`.
Unusable input raises `ValueError`.

The console script is the same interface as the standalone Rust binary:

```sh
csan --cutter-sanborn --cutter-loc \
  --given-name Charles --family-name Dickens --title "Oliver Twist"
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

Publishing to PyPI: `maturin publish -m crates/csan-py/Cargo.toml`
(abi3 wheels for CPython 3.9+; use maturin's manylinux support or
`maturin-action` in CI for portable Linux wheels).

## License

Licensed under either of
[Apache License, Version 2.0](https://github.com/veralvx/csan/blob/main/LICENSE-APACHE)
or [MIT license](https://github.com/veralvx/csan/blob/main/LICENSE-MIT)
at your option.
