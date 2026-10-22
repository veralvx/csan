#!/usr/bin/env node
// The `csan` command of the npm package (`npx @veralvx/csan --help`).
//
// It runs the same command-line interface as the native `csan` binary,
// compiled into the package's WebAssembly module: `runCli` returns the
// exit code and the text to print, which this shim forwards to the process.
// `just npm-build-node` copies it into the package as bin/csan.cjs.

"use strict";

const { runCli } = require("../csan_web.js");

// Whether the native binary would style text on `stream`: clap's terminal
// detection (anstream 1.0's `AutoStream` choice with anstyle-query 1.1),
// applied per stream. `just npm-tty-parity` checks the two still agree.
// Node's `isTTY` does not recognize msys/mintty ptys (Git Bash), where the
// native binary styles but this command does not.
function colorFor(stream) {
  const env = process.env;
  const nonEmpty = (name) => (env[name] ?? "") !== "";
  if (nonEmpty("NO_COLOR")) return false;
  if (nonEmpty("CLICOLOR_FORCE")) return true;
  if (env.CLICOLOR === "0") return false;
  // TERM must be set (except on Windows) and not "dumb".
  const termSupportsColor = env.TERM === undefined
    ? process.platform === "win32"
    : env.TERM !== "dumb";
  return Boolean(stream.isTTY)
    && (termSupportsColor || env.CLICOLOR !== undefined || env.CI !== undefined);
}

// The native binary's write-failure policy: a closed pipe
// (`csan --help | head -1`) keeps the exit code, and any other failure is
// reported in one line and exits 1.
for (const stream of [process.stdout, process.stderr]) {
  stream.on("error", (error) => {
    if (error.code === "EPIPE") return;
    process.exitCode = 1;
    if (stream !== process.stderr) {
      process.stderr.write(`error: failed to write output: ${error.message}\n`);
    }
  });
}

const { code, stdout, stderr } = runCli(process.argv.slice(2), {
  stdout: colorFor(process.stdout),
  stderr: colorFor(process.stderr),
});
// Node exits with `process.exitCode` once the writes have flushed. Empty
// text is not written: the native binary does not touch a stream it has
// nothing for, so a broken but unused stream must not change the outcome.
process.exitCode = code;
if (stdout !== "") process.stdout.write(stdout);
if (stderr !== "") process.stderr.write(stderr);
