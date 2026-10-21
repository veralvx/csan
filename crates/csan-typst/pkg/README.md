# csan (Typst package)

Generate Cutter call numbers for library cataloging, powered by the
[`csan`](https://github.com/veralvx/csan) Rust crate compiled to a Typst
WASM plugin.

```typ
#import "@preview/csan:0.1.0": cutter-sanborn, cutter-loc

#cutter-sanborn(
  forename: "Charles",
  surname: "Dickens",
  title: "Oliver Twist",
) // -> "D548o"

#cutter-loc(surname: "Carter") // -> "C378"
```

Both functions accept `forename`, `surname`, and `title` (at least
one of author/title must be non-blank; blank strings count as absent).
`cutter-sanborn` also takes `prefix-len` and `suffix-len`; `cutter-loc`
takes `max-len` (clamped to 4). Pass `detailed: true` to receive a
dictionary of the parts instead of the formatted string.

Requires Typst 0.14.2 or later. See
[`examples/example.typ`](examples/example.typ) for a compilable demo
document.

## Development

The plugin binary (`csan.wasm`) is built from
[`crates/csan-typst`](https://github.com/veralvx/csan/tree/main/crates/csan-typst)
in the main repository; `tests/test.typ` is its assert-based test suite and
is excluded from the published bundle. When submitting to Typst Universe,
copy this directory to `packages/preview/csan/0.1.0/` (the `tests`
directory need not be copied).

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
