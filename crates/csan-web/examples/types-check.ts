// Type-checks the generated TypeScript declarations (`just npm-types` runs
// `tsc --noEmit --strict` on this file after building the package). It is
// compiled, never run.
//
// Each `@ts-expect-error` must be needed: if a declaration degrades to
// `any`, the line compiles, the directive goes unused, and tsc fails.

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

// The detailed results carry their typed fields.
const prefix: string = sanbornParts.alpha_prefix + locParts.alpha_prefix;
const numeric: number = sanbornParts.numeric + locParts.numeric;
const suffix: string | undefined = sanbornParts.alpha_suffix;
const entry: string = sanbornParts.entry;
const formatted: string = sanbornParts.formatted + locParts.formatted;

// The article options.
const stripped: string = cutterSanborn({
  title: "The Bear",
  lang: "en",
  stripArticle: true,
});
const locStripped: string = cutterLoc({ title: "The Bear", lang: null, stripArticle: null });
const sanbornOff: string = cutterSanborn({ title: "The Bear", lang: null, stripArticle: null });

// runCli takes the arguments and, optionally, the styling.
const result: CliResult = runCli(["--help"]);
const styled: CliResult = runCli(["--method", "dewey"], { stderr: true });
const code: number = result.code + styled.code;
const output: string = result.stdout + styled.stderr;

// The results are not `any`.
// @ts-expect-error: a CliResult is not a string.
const notText: string = runCli([]);
// @ts-expect-error: a call number is not a number.
const notNumber: number = cutterLoc();
// @ts-expect-error: the LoC parts have no work mark.
const noSuffix = cutterLocDetailed().alpha_suffix;
// @ts-expect-error: the Sanborn parts' numeric is a number.
const notNumeric: string = cutterSanbornDetailed().numeric;

// The options are checked.
// @ts-expect-error: a misspelled option key is a type error.
cutterSanborn({ givenName: "Charles" });
// @ts-expect-error: `prefixLen` is not a Library of Congress option.
cutterLoc({ surname: "Carter", prefixLen: 2 });
// @ts-expect-error: `stripArticle` is a boolean.
cutterSanborn({ title: "The Bear", lang: "en", stripArticle: "yes" });
// @ts-expect-error: `stripArticle` is a boolean.
cutterLoc({ title: "The Bear", lang: "en", stripArticle: "yes" });
// @ts-expect-error: `lang` is a string.
cutterSanborn({ title: "The Bear", lang: 1 });
// @ts-expect-error: `lang` is a string.
cutterLoc({ title: "The Bear", lang: 1 });
// @ts-expect-error: unknown styling key.
runCli([], { colour: true });
// @ts-expect-error: the arguments are strings.
runCli([42]);

export {
  code,
  entry,
  formatted,
  loc,
  locParts,
  locStripped,
  noSuffix,
  notNumber,
  notNumeric,
  notText,
  numeric,
  output,
  plain,
  prefix,
  sanborn,
  sanbornOff,
  sanbornParts,
  stripped,
  suffix,
};
