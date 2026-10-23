// Example document for the csan Typst package.
// Compile with: typst compile --root . examples/example.typ (from pkg/)

#import "../lib.typ": cutter-loc, cutter-sanborn

#set page(width: auto, height: auto, margin: 1cm, fill: white)

= Cutter call numbers with csan

#table(
  columns: 4,
  table.header([*Author*], [*Title*], [*Cutter-Sanborn*], [*LoC*]),
  [Charles Dickens], [Oliver Twist],
  [#cutter-sanborn(
      forename: "Charles",
      surname: "Dickens",
      title: "Oliver Twist",
    )],
  [#cutter-loc(surname: "Dickens")],

  [Jorge Luis Borges], [Ficciones],
  [#cutter-sanborn(surname: "Borges", title: "Ficciones")],
  [#cutter-loc(surname: "Borges")],

  [—], [100 best hamburger recipes],
  [#cutter-sanborn(title: "100 best hamburger recipes")],
  [#cutter-loc(title: "100 best hamburger recipes")],
)

A spine label for _Oliver Twist_, classed at Dewey 823.8:

#box(stroke: 0.5pt, inset: 8pt, align(center, text(font: "DejaVu Sans Mono")[
  823.8 \
  #cutter-sanborn(forename: "Charles", surname: "Dickens", title: "Oliver Twist")
]))
