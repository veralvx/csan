// csan — Cutter call numbers (Cutter-Sanborn and Library of Congress).
//
// Thin Typst wrappers around the csan_typst WASM plugin. Both functions
// accept an author (given/family), a title, or both; blank strings count
// as absent. Errors from the plugin surface as Typst diagnostics.

#let _plugin = plugin("csan_typst.wasm")

// Builds the CBOR payload for the plugin, dropping `none` entries.
#let _payload(given-name, family-name, title, ..extra) = {
  let author = (:)
  if given-name != none { author.insert("given_name", str(given-name)) }
  if family-name != none { author.insert("family_name", str(family-name)) }

  let args = (:)
  if author.len() > 0 { args.insert("author", author) }
  if title != none { args.insert("title", str(title)) }
  for (key, value) in extra.named() {
    if value != none { args.insert(key, value) }
  }

  cbor.encode(args)
}

/// Cutter-Sanborn three-figure call number.
///
/// - given-name (str, none): The author's given (first) name.
/// - family-name (str, none): The author's family (last) name.
/// - title (str, none): The work's title. Required when no author is
///   given; with an author it contributes the lowercase work mark.
/// - prefix-len (int, none): Letters taken from the name for the call
///   number prefix (plugin default: 1).
/// - suffix-len (int, none): Letters taken from the title for the work
///   mark (plugin default: 1).
/// - detailed (bool): When true, returns a dictionary with the keys
///   `alpha_prefix`, `numeric`, `alpha_suffix`, `entry`, and `formatted`
///   instead of the formatted string.
/// -> str | dictionary
#let cutter-sanborn(
  given-name: none,
  family-name: none,
  title: none,
  prefix-len: none,
  suffix-len: none,
  detailed: false,
) = {
  let payload = _payload(
    given-name,
    family-name,
    title,
    prefix_len: prefix-len,
    suffix_len: suffix-len,
  )
  if detailed {
    cbor(_plugin.cutter(payload))
  } else {
    str(_plugin.cutter_fmt(payload))
  }
}

/// Library of Congress cutter number.
///
/// - given-name (str, none): The author's given (first) name.
/// - family-name (str, none): The author's family (last) name.
/// - title (str, none): The work's title; used when no author is given.
/// - max-len (int, none): Requested number of digits, clamped to 4 by the
///   plugin (plugin default: 3).
/// - detailed (bool): When true, returns a dictionary with the keys
///   `alpha_prefix`, `numeric`, and `formatted` instead of the formatted
///   string.
/// -> str | dictionary
#let cutter-loc(
  given-name: none,
  family-name: none,
  title: none,
  max-len: none,
  detailed: false,
) = {
  let payload = _payload(
    given-name,
    family-name,
    title,
    max_len: max-len,
  )
  if detailed {
    cbor(_plugin.cutter_loc(payload))
  } else {
    str(_plugin.cutter_loc_fmt(payload))
  }
}
