# csan (Typst package)

Generate Cutter-Sanborn three-figure and Library of Congress cutter numbers
for library cataloging from author names and titles, in Typst documents.
The functions run the Rust [`csan-core`](https://crates.io/crates/csan-core)
library, compiled to a WebAssembly plugin.

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

Both functions accept `forename`, `surname`, and `title` (at least
one of author/title must be non-blank; blank strings count as absent).
`cutter-sanborn` also takes `prefix-len` and `suffix-len`; `cutter-loc`
takes `max-len` (clamped to 4). Pass `detailed: true` to receive a
dictionary of the parts instead of the formatted string.

Requires Typst 0.14.2 or later. See
[`examples/example.typ`](https://github.com/veralvx/csan/blob/main/crates/csan-typst/pkg/examples/example.typ)
for a compilable demo document.

To install, extract the `csan_typst-<version>.tar.gz` attached to each
[GitHub release](https://github.com/veralvx/csan/releases) into
`{data-dir}/typst/packages/local/csan/<version>/` (see
[local packages](https://github.com/typst/packages#local-packages)), then
import `@local/csan:<version>`.

## Development

The plugin binary (`csan_typst.wasm`) is built from
[`crates/csan-typst`](https://github.com/veralvx/csan/tree/main/crates/csan-typst)
in the main repository; `tests/test.typ` is its assert-based test suite and
is excluded from the published bundle. When submitting to Typst Universe,
copy this directory to `packages/preview/csan/<version>/` (the `tests`
directory need not be copied) and import it as `@preview/csan:<version>`.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
