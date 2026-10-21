// Example document for the csan Typst package.
// Compile with: typst compile --root .. examples/example.typ (from pkg/)

#import "../lib.typ": cutter-sanborn, cutter-loc

#set page(width: auto, height: auto, margin: 1cm)

= Cutter call numbers with csan

#table(
  columns: 4,
  table.header([*Author*], [*Title*], [*Cutter-Sanborn*], [*LoC*]),
  [Charles Dickens], [Oliver Twist],
  [#cutter-sanborn(
      given-name: "Charles",
      family-name: "Dickens",
      title: "Oliver Twist",
    )],
  [#cutter-loc(family-name: "Dickens")],

  [Jorge Luis Borges], [Ficciones],
  [#cutter-sanborn(family-name: "Borges", title: "Ficciones")],
  [#cutter-loc(family-name: "Borges")],

  [—], [100 best hamburger recipes],
  [#cutter-sanborn(title: "100 best hamburger recipes")],
  [#cutter-loc(title: "100 best hamburger recipes")],
)

The full call number for a work shelved under QA76.73:
`QA76.73 .#cutter-sanborn(given-name: "Graydon", family-name: "Hoare")`
