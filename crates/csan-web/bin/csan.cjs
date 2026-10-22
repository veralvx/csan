#!/usr/bin/env node
// The `csan` command of the npm package (`npx @veralvx/csan --help`).
//
// It runs the same command-line interface as the native `csan` binary,
// compiled into the package's WebAssembly module: `runCli` returns the
// exit code and the text to print, which this shim forwards to the process.
// `just npm-build-node` copies it into the package as bin/csan.cjs.

"use strict";

const { runCli } = require("../csan_web.js");

// A closed pipe (e.g. `csan --help | head -1`) is not an error worth a
// stack trace; any other stream error still is.
for (const stream of [process.stdout, process.stderr]) {
  stream.on("error", (error) => {
    if (error.code !== "EPIPE") throw error;
  });
}

const { code, stdout, stderr } = runCli(process.argv.slice(2));
process.stdout.write(stdout);
process.stderr.write(stderr);
// Let pending writes flush before Node exits with the code.
process.exitCode = code;
