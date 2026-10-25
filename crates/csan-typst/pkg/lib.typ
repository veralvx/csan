// csan — Cutter call numbers (Cutter-Sanborn and Library of Congress).
//
// Thin Typst wrappers around the csan_typst WASM plugin. Both functions
// accept an author (forename, surname, or both), a title, or both; blank
// strings count as absent. With `strip-article: true`, the title is filed
// without its initial article in the language `lang` names. Errors from the
// plugin surface as Typst diagnostics.

#let _plugin = plugin("csan_typst.wasm")

// Builds the CBOR payload for the plugin, dropping `none` entries.
#let _payload(forename, surname, title, ..extra) = {
  let author = (:)
  if forename != none { author.insert("forename", str(forename)) }
  if surname != none { author.insert("surname", str(surname)) }

  let args = (:)
  if author.len() > 0 { args.insert("author", author) }
  if title != none { args.insert("title", str(title)) }
  for (key, value) in extra.named() {
    if value != none { args.insert(key, value) }
  }

  cbor.encode(args)
}

// Checks the types of `lang` and `strip-article` the way Typst's own
// functions do, so that a mistake names its argument. (The plugin rejects
// it too, without saying which.)
#let _check-language(lang, strip-article) = {
  assert(
    lang == none or type(lang) == str,
    message: "`lang`: expected string or none, found " + str(type(lang)),
  )
  assert(
    type(strip-article) == bool,
    message: "`strip-article`: expected boolean, found "
      + str(type(strip-article)),
  )
}

/// Cutter-Sanborn three-figure call number.
///
/// - forename (str, none): The author's forename.
/// - surname (str, none): The author's surname.
/// - title (str, none): The work's title. Required when no author is
///   given; with an author it contributes the lowercase work mark.
/// - prefix-len (int, none): Letters taken from the name for the call
///   number prefix (plugin default: 1).
/// - suffix-len (int, none): Letters taken from the title for the work
///   mark (plugin default: 1).
/// - lang (str, none): The work's language, for `strip-article`: an ISO 639
///   code, a BCP 47 tag or a POSIX locale, case-insensitive (`"en"`,
///   `"pt-BR"`, ...). Alone it changes nothing, and a blank one counts as
///   absent; an unknown one is an error, even with `strip-article` off.
/// - strip-article (bool): When true, files the title without its initial
///   article in `lang` (_The Bear_ under _Bear_). Requires `lang`. Author
///   names are never stripped.
/// - detailed (bool): When true, returns a dictionary with the keys
///   `alpha_prefix`, `numeric`, `alpha_suffix`, `entry`, and `formatted`
///   instead of the formatted string.
/// -> str | dictionary
#let cutter-sanborn(
  forename: none,
  surname: none,
  title: none,
  prefix-len: none,
  suffix-len: none,
  lang: none,
  strip-article: false,
  detailed: false,
) = {
  _check-language(lang, strip-article)
  let payload = _payload(
    forename,
    surname,
    title,
    prefix_len: prefix-len,
    suffix_len: suffix-len,
    lang: lang,
    strip_article: strip-article,
  )
  if detailed {
    cbor(_plugin.cutter(payload))
  } else {
    str(_plugin.cutter_fmt(payload))
  }
}

/// Library of Congress cutter number.
///
/// - forename (str, none): The author's forename.
/// - surname (str, none): The author's surname.
/// - title (str, none): The work's title; used when no author is given.
/// - max-len (int, none): Requested number of digits, clamped to 4 by the
///   plugin (plugin default: 3).
/// - lang (str, none): The work's language, for `strip-article`: an ISO 639
///   code, a BCP 47 tag or a POSIX locale, case-insensitive (`"en"`,
///   `"pt-BR"`, ...). Alone it changes nothing, and a blank one counts as
///   absent; an unknown one is an error, even with `strip-article` off.
/// - strip-article (bool): When true, files the title without its initial
///   article in `lang` (_The Bear_ under _Bear_). Requires `lang`. Author
///   names are never stripped.
/// - detailed (bool): When true, returns a dictionary with the keys
///   `alpha_prefix`, `numeric`, and `formatted` instead of the formatted
///   string.
/// -> str | dictionary
#let cutter-loc(
  forename: none,
  surname: none,
  title: none,
  max-len: none,
  lang: none,
  strip-article: false,
  detailed: false,
) = {
  _check-language(lang, strip-article)
  let payload = _payload(
    forename,
    surname,
    title,
    max_len: max-len,
    lang: lang,
    strip_article: strip-article,
  )
  if detailed {
    cbor(_plugin.cutter_loc(payload))
  } else {
    str(_plugin.cutter_loc_fmt(payload))
  }
}
