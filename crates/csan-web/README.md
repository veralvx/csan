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
`sideEffects`, and the license/repository metadata from `Cargo.toml`):

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack        # or: npm i -g wasm-pack

# Bundler-oriented package (webpack, Vite, Rollup, …) in ./pkg
wasm-pack build --release --target bundler

# Node-oriented package in ./pkg-node (`just npm-test` from the
# repository root also runs the demo and smoke tests, and
# `just npm-pack` packs the release tarball)
wasm-pack build --release --target nodejs --out-dir pkg-node

# Plain-browser ES module package in ./pkg-web
wasm-pack build --release --target web --out-dir pkg-web
```

If you run `wasm-bindgen` by hand instead, its CLI version must exactly
match the `wasm-bindgen` dependency version in `Cargo.lock`.

## Running the examples

```sh
wasm-pack build --release --target nodejs --out-dir pkg-node
node examples/node-demo.cjs
node examples/smoke-test.cjs

wasm-pack build --release --target web --out-dir pkg-web
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
