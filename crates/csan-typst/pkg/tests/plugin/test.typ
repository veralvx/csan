// Assert-based test suite for the csan Typst package: a tytanic
// compile-only test (`just typst test`), which passes when every assert
// holds.

#import "/lib.typ": cutter-loc, cutter-sanborn

// --- Cutter-Sanborn: formatted strings -------------------------------
#assert.eq(
  cutter-sanborn(forename: "Charles", surname: "Dickens"),
  "D548",
)
#assert.eq(
  cutter-sanborn(
    forename: "Charles",
    surname: "Dickens",
    title: "Oliver Twist",
  ),
  "D548o",
)
#assert.eq(
  cutter-sanborn(
    forename: "Charles",
    surname: "Dickens",
    title: "Oliver Twist",
    prefix-len: 2,
    suffix-len: 3,
  ),
  "Di548oli",
)
#assert.eq(
  cutter-sanborn(surname: "Borges", title: "Ficciones"),
  "B732f",
)
#assert.eq(
  cutter-sanborn(forename: "VERA", surname: "LVX"),
  "L979",
)
// Title-only entries are cuttered by title.
#assert.eq(cutter-sanborn(title: "Bear") != "", true)

// --- Cutter-Sanborn: detailed dictionaries ---------------------------
#let detailed = cutter-sanborn(
  forename: "Charles",
  surname: "Dickens",
  title: "Oliver Twist",
  detailed: true,
)
#assert.eq(detailed.alpha_prefix, "D")
#assert.eq(detailed.numeric, 548)
#assert.eq(detailed.alpha_suffix, "o")
#assert.eq(detailed.entry, "Dicke")
#assert.eq(detailed.formatted, "D548o")

#let no-mark = cutter-sanborn(surname: "Borges", detailed: true)
#assert.eq(no-mark.alpha_suffix, none)
#assert.eq(no-mark.entry, "Borg")
#assert.eq(no-mark.formatted, "B732")

// --- Library of Congress ---------------------------------------------
#assert.eq(cutter-loc(surname: "Carter"), "C378")
#assert.eq(cutter-loc(surname: "Macdonald", max-len: 4), "M3336")
#assert.eq(cutter-loc(title: "100 best hamburger recipes"), "A153")

#let loc-detailed = cutter-loc(surname: "Carter", detailed: true)
#assert.eq(loc-detailed.alpha_prefix, "C")
#assert.eq(loc-detailed.numeric, 378)
#assert.eq(loc-detailed.formatted, "C378")

// --- Blank author falls back to the title ----------------------------
#assert.eq(cutter-loc(surname: "   ", title: "Bear"), "B437")

// --- Initial articles (lang, strip-article) --------------------------
// Every stripped value differs from the unstripped one, so a plugin
// that ignores these arguments fails here.
#assert.eq(
  cutter-sanborn(
    forename: "Charles",
    surname: "Dickens",
    title: "A Christmas Carol",
    lang: "en",
    strip-article: true,
  ),
  "D548c",
)
#assert.eq(
  cutter-sanborn(title: "The Bear", lang: "EN-us", strip-article: true),
  "B368",
)
#assert.eq(
  cutter-loc(title: "The Bear", lang: "en", strip-article: true),
  "B437",
)
#assert.eq(
  cutter-loc(title: "The 100 best recipes", lang: "en", strip-article: true),
  "A153",
)

#let stripped = cutter-sanborn(
  title: "The Bear",
  lang: "en",
  strip-article: true,
  detailed: true,
)
#assert.eq(stripped.entry, "Bear")
#assert.eq(stripped.formatted, "B368")
#assert.eq(
  cutter-loc(
    title: "The Bear",
    lang: "en",
    strip-article: true,
    detailed: true,
  ).formatted,
  "B437",
)

// Author names are never stripped: only the title loses its article.
#assert.eq(
  cutter-sanborn(
    forename: "Jean de",
    surname: "La Fontaine",
    title: "Les Fables",
    lang: "fr",
    strip-article: true,
  ),
  "L166f",
)

// A language alone, or a blank one with the switch off, changes nothing.
#assert.eq(cutter-sanborn(title: "The Bear", lang: "en"), "T375")
#assert.eq(
  cutter-loc(title: "The Bear", lang: "en", strip-article: false),
  "T443",
)
#assert.eq(cutter-loc(title: "The Bear", lang: "  "), "T443")

// The language set once, with `.with`, and the switch per call.
#let cutter-pt = cutter-sanborn.with(lang: "pt")
#assert.eq(cutter-pt(surname: "Alencar", title: "O Guarani"), "A368o")
#assert.eq(
  cutter-pt(surname: "Alencar", title: "O Guarani", strip-article: true),
  "A368g",
)
#let loc-pt = cutter-loc.with(lang: "pt", strip-article: true)
#assert.eq(loc-pt(title: "O Guarani"), "G837")

// The document's language, through `context`.
#set text(lang: "pt")
#context assert.eq(
  cutter-sanborn(
    surname: "Alencar",
    title: "O Guarani",
    lang: text.lang,
    strip-article: true,
  ),
  "A368g",
)
#set text(lang: "en")

// Every code the README lists is accepted ("Bear" has no article).
#for code in (
  "af",
  "sq",
  "br",
  "ca",
  "da",
  "nl",
  "en",
  "eo",
  "fr",
  "gl",
  "de",
  "hu",
  "is",
  "ga",
  "it",
  "la",
  "lb",
  "mt",
  "no",
  "oc",
  "pt",
  "ro",
  "es",
  "sv",
  "cy",
) {
  assert.eq(cutter-loc(title: "Bear", lang: code, strip-article: true), "B437")
}

= csan plugin test suite
All assertions passed.
