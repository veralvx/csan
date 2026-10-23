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

= csan plugin test suite
All assertions passed.
