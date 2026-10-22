# csan-web

Cutter call number generation for JavaScript, as a WebAssembly npm package
built from the [`csan-core`](https://github.com/veralvx/csan) Rust crate
with `wasm-bindgen`.

```js
import { cutterLoc, cutterSanborn, cutterSanbornDetailed } from "@veralvx/csan";

cutterSanborn({
  forename: "Charles",
  surname: "Dickens",
  title: "Oliver Twist",
}); // "D548o"
cutterLoc({ surname: "Carter" }); // "C378"
cutterSanbornDetailed({ surname: "Borges" });
// { alpha_prefix: "B", numeric: 732, alpha_suffix: undefined,
//   entry: "Borg", formatted: "B732" }
```

Every function takes one options object with camelCase keys: `forename`,
`surname`, `title`, plus `prefixLen`/`suffixLen` for Cutter-Sanborn and
`maxLen` (clamped to 4) for the Library of Congress method. At least one of
the name/title inputs must be non-blank; blank strings count as absent.
Only the object's own properties are read, and unknown keys are rejected (a
misspelled key, or `prefixLen` passed to `cutterLoc`, throws instead of
being silently ignored). Failures throw a
JavaScript `Error`. The package ships TypeScript declarations for the
options (`SanbornOptions`, `LocOptions`) and the detailed results. The `*Detailed` variants return the
cutter's parts as a plain object whose keys (`alpha_prefix`, `numeric`,
`alpha_suffix`, `entry`, `formatted`) match the shared representation used by
the other `csan` frontends.

## Command line

The package provides the `csan` command, the same command-line interface as
the native `csan` binary (options, help, messages, and exit codes), compiled
into the package's WebAssembly module:

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

The command makes the WebAssembly module larger: the package is about
210 KB gzipped, up from about 130 KB without it.

## Installing

The package is published to npm as
[`@veralvx/csan`](https://www.npmjs.com/package/@veralvx/csan), built for
Node (CommonJS, with TypeScript declarations):

```sh
npm install @veralvx/csan
```

Each [GitHub release](https://github.com/veralvx/csan/releases) also
carries the same tarball, `veralvx-csan-<version>.tgz`.

## Building the npm package

The package is produced with [`wasm-pack`](https://rustwasm.github.io/wasm-pack/),
which compiles the crate, runs `wasm-bindgen`, and generates a
conventional npm package (`package.json` with `name`, `types`, `files`,
`sideEffects`, and the license/repository metadata from `Cargo.toml`).
Build the published package (the Node build) with the `just` recipes, from
the repository root; besides running `wasm-pack`, they optimize for size,
add the `csan` command, and set the npm name:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack        # or: npm i -g wasm-pack

just npm-build-node   # the npm package, in crates/csan-web/pkg-node
just npm-test         # ... then run the demo, smoke, and type tests
just npm-tty-parity   # ... compare its command with the native binary
just npm-pack         # ... then pack the release tarball

# Other targets (no `csan` command, though `runCli` is still compiled in):
just npm-build        # bundlers, in crates/csan-web/pkg
```

For a plain-browser ES module, run in this directory (`opt-level` keeps
the module as small as the npm build):

```sh
wasm-pack build --release --target web --out-dir pkg-web \
  -- --config 'profile.release.opt-level="z"'
```

If you run `wasm-bindgen` by hand instead, its CLI version must exactly
match the `wasm-bindgen` dependency version in `Cargo.lock`.

## Running the examples

```sh
# From the repository root: build the package, then run
# examples/node-demo.cjs and examples/smoke-test.cjs against it.
just npm-test

# The browser demo, from this directory (build pkg-web as above):
python3 -m http.server   # then open http://localhost:8000/examples/web/
```

## Publishing to npm

Releases publish automatically: the release workflow builds and
smoke-tests the Node package with `just npm-pack` (which names it
`@veralvx/csan`; wasm-pack would default to the crate name), and
`.github/workflows/publish-npm.yml` publishes that tarball once the release
succeeds. The `version` comes from `Cargo.toml`. To inspect the package
locally, run `just npm-pack` and `npm publish --dry-run` on the tarball.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE)
or [MIT license](LICENSE-MIT) at your option.
