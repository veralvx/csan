// The errors of `lang` and `strip-article`: a tytanic compile-only test
// (`just typst test`), which uses tytanic's `catch`, so plain `typst
// compile` can't run it. Each assertion checks the distinctive part of the
// message: the plugin's unit tests pin the whole text, and Typst adds its
// own prefix.

#import "/lib.typ": cutter-loc, cutter-sanborn

// Asserts that `call` fails with a message containing `part`.
#let fails-with(call, part) = {
  let message = catch(call)
  assert(
    message != none and message.contains(part),
    message: "expected an error containing "
      + repr(part)
      + ", got "
      + repr(message),
  )
}

// Every case through each function, plain and detailed (all four plugin
// functions).
#for call in (
  cutter-sanborn,
  cutter-sanborn.with(detailed: true),
  cutter-loc,
  cutter-loc.with(detailed: true),
) {
  // An unknown code is an error whether the switch is off or on, and
  // before a missing input.
  fails-with(
    () => call(title: "The Bear", lang: "xx"),
    "unknown language \"xx\"",
  )
  fails-with(
    () => call(title: "The Bear", lang: "xx", strip-article: true),
    "unknown language \"xx\"",
  )
  fails-with(() => call(lang: "xx"), "unknown language \"xx\"")

  // A partial application with a bad code fails on every call.
  let call-xx = call.with(lang: "xx")
  fails-with(() => call-xx(title: "The Bear"), "unknown language \"xx\"")

  // The switch needs a language, and a blank one counts as none.
  for lang in (none, "", "  ") {
    fails-with(
      () => call(title: "The Bear", lang: lang, strip-article: true),
      "`strip-article` requires `lang`",
    )
  }

  // The arguments are typed, as in Typst's own functions: no coercion, and
  // no `none` for the switch. The message names the argument.
  for (value, found) in (("yes", "string"), (1, "integer"), (none, "none")) {
    fails-with(
      () => call(title: "The Bear", lang: "en", strip-article: value),
      "`strip-article`: expected boolean, found " + found,
    )
  }
  for (value, found) in ((5, "integer"), (true, "boolean")) {
    fails-with(
      () => call(title: "The Bear", lang: value),
      "`lang`: expected string or none, found " + found,
    )
  }
}
