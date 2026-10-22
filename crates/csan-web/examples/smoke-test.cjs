// Smoke test for the Node build of csan-web, run by `just npm-test` and CI.
//
//     wasm-pack build --release --target nodejs --out-dir pkg-node
//     node examples/smoke-test.cjs

const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const path = require("node:path");
const {
  cutterSanborn,
  cutterSanbornDetailed,
  cutterLoc,
  cutterLocDetailed,
  runCli,
} = require("../pkg-node/csan_web.js");

const dickens = {
  forename: "Charles",
  surname: "Dickens",
  title: "Oliver Twist",
};

assert.equal(cutterSanborn(dickens), "D548o");
assert.equal(cutterLoc(dickens), "D535");
assert.equal(
  cutterSanborn({ ...dickens, prefixLen: 2, suffixLen: 3 }),
  "Di548oli",
);
assert.equal(cutterLoc({ surname: "Macdonald", maxLen: 64 }), "M3336");
assert.equal(cutterLoc({ title: "100 best recipes" }), "A153");

const sanborn = cutterSanbornDetailed({ surname: "Borges" });
assert.equal(sanborn.alpha_prefix, "B");
assert.equal(sanborn.numeric, 732);
assert.equal(sanborn.alpha_suffix, undefined);
assert.equal(sanborn.entry, "Borg");
assert.equal(sanborn.formatted, "B732");
assert.deepEqual(cutterLocDetailed({ surname: "Carter" }), {
  alpha_prefix: "C",
  numeric: 378,
  formatted: "C378",
});

// Missing or blank input, and unknown keys (e.g. a typo, or a Cutter-Sanborn
// knob passed to the LoC method), throw instead of returning a wrong number.
assert.throws(() => cutterSanborn(), /forename, surname, or title/);
assert.throws(() => cutterLoc({ surname: "   " }), /surname/);
assert.throws(
  () => cutterSanborn({ givenName: "Charles", title: "Oliver Twist" }),
  /givenName/,
);
assert.throws(() => cutterLoc({ surname: "Carter", prefixLen: 2 }), /prefixLen/);
assert.throws(() => cutterLoc("Carter"), /options must be an object/);
assert.throws(
  () => cutterLoc({ surname: "Carter", maxLen: 2.5 }),
  (e) => e.message.startsWith("invalid type"),
);

// runCli: the native CLI's interface, with the output captured.
assert.deepEqual(runCli(["--surname", "Dickens", "--title", "Oliver Twist"]), {
  code: 0,
  stdout: "D548o\n",
  stderr: "",
});
assert.equal(runCli(["--method=cutter-loc", "--surname", "Carter"]).stdout, "C378\n");

// The package's `csan` command (bin/csan.cjs), run as a process.
const bin = path.join(__dirname, "..", "pkg-node", "bin", "csan.cjs");
const csan = (...args) => spawnSync(process.execPath, [bin, ...args], { encoding: "utf8" });

let run = csan("--method", "cutter-loc", "--surname", "Dickens");
assert.equal(run.status, 0);
assert.equal(run.stdout, "D535\n");
assert.equal(run.stderr, "");

run = csan("--help");
assert.equal(run.status, 0);
assert.match(run.stdout, /--method <METHOD>/);

run = csan("--version");
assert.equal(run.status, 0);
assert.match(run.stdout, /^csan \d+\.\d+\.\d+\n$/);

run = csan("--method", "dewey");
assert.equal(run.status, 2, "usage error");
assert.equal(run.stdout, "");
assert.match(run.stderr, /possible values: cutter-sanborn, cutter-loc/);

run = csan();
assert.equal(run.status, 1, "runtime error");
assert.match(run.stderr, /provide at least one non-blank input/);

console.log("csan-web node smoke test passed");
