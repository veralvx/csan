// Type-checks the generated TypeScript declarations (`just npm-test` runs
// `tsc --noEmit --strict` on this file after building the package). It is
// compiled, never run.

import {
  type CliResult,
  cutterLoc,
  type CutterLocDetailed,
  cutterLocDetailed,
  cutterSanborn,
  type CutterSanbornDetailed,
  cutterSanbornDetailed,
  runCli,
} from "../pkg-node/csan_web.js";

// Every options argument is optional, as in JavaScript.
const plain: string = cutterSanborn();
const sanborn: string = cutterSanborn({ surname: "Dickens", prefixLen: 2 });
const loc: string = cutterLoc({ title: "100 best recipes", maxLen: null });
const sanbornParts: CutterSanbornDetailed = cutterSanbornDetailed({
  surname: "Borges",
});
const locParts: CutterLocDetailed = cutterLocDetailed(null);

// runCli takes the arguments and, optionally, the styling.
const result: CliResult = runCli(["--help"]);
const styled: CliResult = runCli(["--method", "dewey"], { stderr: true });
const code: number = result.code + styled.code;

// @ts-expect-error: a misspelled option key is a type error.
cutterSanborn({ givenName: "Charles" });
// @ts-expect-error: `prefixLen` is not a Library of Congress option.
cutterLoc({ surname: "Carter", prefixLen: 2 });
// @ts-expect-error: unknown styling key.
runCli([], { colour: true });

export { code, loc, locParts, plain, sanborn, sanbornParts };
