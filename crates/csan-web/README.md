# @veralvx/csan

Generate Cutter-Sanborn three-figure and Library of Congress cutter numbers
for library cataloging from author names and titles, in Node. The package
is the Rust [`csan-core`](https://crates.io/crates/csan-core) library
compiled to WebAssembly, with TypeScript declarations and the `csan`
command-line tool.

## Install

```sh
npm install @veralvx/csan
```

Or run the command without installing: `npx @veralvx/csan --help`. The
package is built for Node (CommonJS, which `import` also loads). Each
[GitHub release](https://github.com/veralvx/csan/releases) carries the
same tarball, `veralvx-csan-<version>.tgz`.

## Usage

```js
const { cutterLoc, cutterSanborn, cutterSanbornDetailed } = require("@veralvx/csan");

cutterSanborn({
  forename: "Charles",
  surname: "Dickens",
  title: "Oliver Twist",
}); // "D548o"
cutterLoc({ surname: "Carter" }); // "C378"
cutterSanbornDetailed({ surname: "Borges" });
// { alpha_prefix: "B", alpha_suffix: undefined, entry: "Borg",
//   numeric: 732, formatted: "B732" }
```

Every function takes one options object with camelCase keys: `forename`,
`surname`, `title`, plus `prefixLen`/`suffixLen` for Cutter-Sanborn and
`maxLen` (clamped to 4) for the Library of Congress method. At least one of
the name/title inputs must be non-blank; blank strings count as absent.

- Only the object's own properties are read, and unknown keys are rejected:
  a misspelled key, or `prefixLen` passed to `cutterLoc`, throws instead of
  being silently ignored.
- Failures throw a JavaScript `Error`.
- The `*Detailed` variants (`cutterSanbornDetailed`, `cutterLocDetailed`)
  return the cutter's parts as a plain object whose keys (`alpha_prefix`,
  `numeric`, `alpha_suffix`, `entry`, `formatted`) match the other `csan`
  frontends.
- The TypeScript declarations type the options (`SanbornOptions`,
  `LocOptions`) and the detailed results.

## Command line

The `csan` command has the same interface as the native `csan` binary
(options, help, messages, and exit codes):

```sh
npx @veralvx/csan --forename Charles --surname Dickens --title "Oliver Twist"   # D548o
npx @veralvx/csan --method cutter-loc --surname Carter                         # C378
npx @veralvx/csan --help
```

Output is styled exactly when the native binary would style it (on a
color-capable terminal, honoring `NO_COLOR`, `CLICOLOR`, and
`CLICOLOR_FORCE`; Node does not detect Git Bash/mintty terminals, so output
there is plain). Write failures follow the same rules: a closed pipe is not
an error, and any other failure prints `error: failed to write output: ...`
and exits 1 (the detail is the platform's error description, worded by Rust
or Node respectively).

`runCli(args, styling?)` exposes the same interface to code: it takes the
arguments (without the program name) and returns `{ code, stdout, stderr }`
instead of printing; `styling` (`{ stdout?, stderr? }`) asks for ANSI
styles on either stream.

The package is about 210 KB gzipped, of which the command-line interface
accounts for about 80 KB.

## Building from source

The package is built from
[`crates/csan-web`](https://github.com/veralvx/csan/tree/main/crates/csan-web)
with [`wasm-pack`](https://rustwasm.github.io/wasm-pack/), through the
`just` recipes at the repository root; besides running `wasm-pack`, they
optimize for size, add the `csan` command, and set the npm name:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack        # or: npm i -g wasm-pack

just npm-build-node   # the npm package, in crates/csan-web/pkg-node
just npm-test         # ... then run the demo and smoke tests
just npm-types        # ... type-check its TypeScript declarations
just npm-tty-parity   # ... compare its command with the native binary
just npm-pack         # ... then pack the release tarball

# Other targets (no `csan` command, though `runCli` is still compiled in):
just npm-build        # bundlers, in crates/csan-web/pkg
```

For a plain-browser ES module, run in `crates/csan-web` (`opt-level` keeps
the module as small as the npm build):

```sh
wasm-pack build --release --target web --out-dir pkg-web \
  -- --config 'profile.release.opt-level="z"'
python3 -m http.server   # then open http://localhost:8000/examples/web/
```

If you run `wasm-bindgen` by hand instead, its CLI version must exactly
match the `wasm-bindgen` dependency version in `Cargo.lock`.

Releases publish automatically: the Release workflow packs the Node package
with `just npm-pack`, and `publish-npm.yml` publishes that tarball once the
release succeeds (see
[CONTRIBUTING.md](https://github.com/veralvx/csan/blob/main/CONTRIBUTING.md#releasing)).

## License

Licensed under either of
[Apache License, Version 2.0](https://github.com/veralvx/csan/blob/main/LICENSE-APACHE)
or [MIT license](https://github.com/veralvx/csan/blob/main/LICENSE-MIT)
at your option.
