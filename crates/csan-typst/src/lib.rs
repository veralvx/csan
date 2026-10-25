#![forbid(unsafe_code)]

//! Typst WASM plugin exposing [`csan_core`]'s Cutter call number generation.
//!
//! Compiled to `wasm32-unknown-unknown`, this crate speaks the [Typst plugin
//! protocol](https://typst.app/docs/reference/foundations/plugin/#protocol)
//! and exports four functions, each taking a single CBOR-encoded argument
//! map:
//!
//! | export           | returns                                          |
//! |------------------|--------------------------------------------------|
//! | `cutter`         | CBOR map of the [`CutterSanborn`] parts + `formatted` |
//! | `cutter_fmt`     | the Cutter-Sanborn call number as a UTF-8 string |
//! | `cutter_loc`     | CBOR map of the [`CutterLoc`] parts + `formatted` |
//! | `cutter_loc_fmt` | the `LoC` cutter number as a UTF-8 string        |
//!
//! The argument map accepts `title` (string), `author` (map with optional
//! `forename`/`surname`), and the optional tuning knobs `prefix_len`,
//! `suffix_len`, and `max_len` (see [`CutterConfig`]), plus `lang` (a
//! language code, see [`Language`]) and `strip_article` (bool): with
//! `strip_article` true, the title is filed without its initial article in
//! `lang` (see [`csan_core::Opus::with_article_language`]). At least one of
//! `author`/`title` must be usable. Any other top-level key is an error.
//! Errors are reported as protocol error strings, which Typst surfaces as
//! diagnostics.
//!
//! On non-wasm targets the same functions compile as ordinary library
//! functions, which is how the unit tests exercise the exact plugin surface.

use ciborium::{de::from_reader, ser::into_writer};
use csan_core::{
    Attribution, Author, CutterConfig, CutterGenerator, CutterLoc, CutterLocMethod, CutterSanborn,
    CutterSanbornMethod, Detailed, Language, MissingLanguage,
};
use serde::Deserialize;

#[cfg(target_arch = "wasm32")]
wasm_minimal_protocol::initiate_protocol!();

/// The CBOR argument map shared by every plugin function.
///
/// Unknown keys are rejected, so that a plugin older than `lib.typ` fails
/// instead of silently ignoring an argument it doesn't know.
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct EntryArgs {
    /// The work's title; enables title-derived cutters and work marks.
    #[serde(default)]
    title: Option<String>,
    /// The work's author.
    #[serde(default)]
    author: Option<Author>,
    /// Overrides [`CutterConfig::prefix_len`] when present.
    #[serde(default)]
    prefix_len: Option<usize>,
    /// Overrides [`CutterConfig::suffix_len`] when present.
    #[serde(default)]
    suffix_len: Option<usize>,
    /// Overrides [`CutterConfig::max_len`] when present.
    #[serde(default)]
    max_len: Option<usize>,
    /// The work's language, for `strip_article`; blank counts as absent.
    #[serde(default)]
    lang: Option<String>,
    /// Files the title without its initial article in `lang`.
    #[serde(default)]
    strip_article: bool,
}

impl EntryArgs {
    /// Decodes the CBOR argument map.
    fn parse(arg: &[u8]) -> Result<Self, String> {
        from_reader(arg).map_err(|e| format!("failed to parse arguments: {e}"))
    }

    /// Builds the generation config, applying any per-call overrides.
    fn config(&self) -> CutterConfig {
        CutterConfig::from_overrides(self.prefix_len, self.suffix_len, self.max_len)
    }

    /// The language the title is filed in: `lang` when `strip_article` is
    /// on, through [`Language::from_optional`] and [`Language::for_rule`].
    /// An unknown `lang` is an error even with the switch off.
    fn article_language(&self) -> Result<Option<Language>, String> {
        let language = Language::from_optional(self.lang.as_deref()).map_err(|e| e.to_string())?;
        Language::for_rule(language, self.strip_article)
            .map_err(|MissingLanguage| String::from("`strip-article` requires `lang`"))
    }

    /// Resolves the argument map into an [`Attribution`], with its title
    /// filed in [`Self::article_language`], whose errors come first.
    ///
    /// Delegates to [`Attribution::from_parts`], so blank strings are
    /// treated as absent — per field: `{"author": {"surname": " ",
    /// "forename": "Prince"}}` cutters on the forename, and
    /// `{"author": {"surname": " "}, "title": "Dune"}` falls back to a
    /// title-derived cutter instead of erroring.
    fn attribution(self) -> Result<Attribution, String> {
        let language = self.article_language()?;
        let author = self.author.as_ref();
        let entry = Attribution::from_parts(
            author.and_then(Author::forename),
            author.and_then(Author::surname),
            self.title.as_deref(),
        )
        .ok_or_else(|| {
            String::from("at least one of `author` or `title` must be provided and non-blank")
        })?;
        Ok(entry.with_article_language(language))
    }
}

/// Generates the [`CutterSanborn`] value for a CBOR argument map.
fn sanborn_from(arg: &[u8]) -> Result<CutterSanborn, String> {
    let args = EntryArgs::parse(arg)?;
    let config = args.config();
    CutterSanbornMethod::generate(&args.attribution()?, config)
        .map_err(|e| format!("cutter generation failed: {e}"))
}

/// Generates the [`CutterLoc`] value for a CBOR argument map.
fn loc_from(arg: &[u8]) -> Result<CutterLoc, String> {
    let args = EntryArgs::parse(arg)?;
    let config = args.config();
    CutterLocMethod::generate(&args.attribution()?, config)
        .map_err(|e| format!("cutter generation failed: {e}"))
}

/// CBOR-encodes a serializable value into the protocol result buffer.
fn to_cbor(value: &impl serde::Serialize) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    into_writer(value, &mut out).map_err(|e| format!("failed to serialize response: {e}"))?;
    Ok(out)
}

/// Returns the Cutter-Sanborn number as a CBOR map with the keys
/// `alpha_prefix`, `numeric`, `alpha_suffix`, `entry`, and `formatted`.
///
/// # Errors
/// Returns a protocol error string when the arguments cannot be decoded,
/// name the wrong fields, give an unknown `lang` or `strip_article` without
/// one, or fail Cutter generation.
#[cfg_attr(target_arch = "wasm32", wasm_minimal_protocol::wasm_func)]
pub fn cutter(arg: &[u8]) -> Result<Vec<u8>, String> {
    to_cbor(&Detailed::new(&sanborn_from(arg)?))
}

/// Returns the Cutter-Sanborn call number as a UTF-8 string
/// (e.g. `D548o`).
///
/// # Errors
/// Returns a protocol error string when the arguments cannot be decoded,
/// name the wrong fields, give an unknown `lang` or `strip_article` without
/// one, or fail Cutter generation.
#[cfg_attr(target_arch = "wasm32", wasm_minimal_protocol::wasm_func)]
pub fn cutter_fmt(arg: &[u8]) -> Result<Vec<u8>, String> {
    Ok(sanborn_from(arg)?.to_string().into_bytes())
}

/// Returns the Library of Congress cutter number as a CBOR map with the
/// keys `alpha_prefix`, `numeric`, and `formatted`.
///
/// # Errors
/// Returns a protocol error string when the arguments cannot be decoded,
/// name the wrong fields, give an unknown `lang` or `strip_article` without
/// one, or fail Cutter generation.
#[cfg_attr(target_arch = "wasm32", wasm_minimal_protocol::wasm_func)]
pub fn cutter_loc(arg: &[u8]) -> Result<Vec<u8>, String> {
    to_cbor(&Detailed::new(&loc_from(arg)?))
}

/// Returns the Library of Congress cutter number as a UTF-8 string
/// (e.g. `D535`).
///
/// # Errors
/// Returns a protocol error string when the arguments cannot be decoded,
/// name the wrong fields, give an unknown `lang` or `strip_article` without
/// one, or fail Cutter generation.
#[cfg_attr(target_arch = "wasm32", wasm_minimal_protocol::wasm_func)]
pub fn cutter_loc_fmt(arg: &[u8]) -> Result<Vec<u8>, String> {
    Ok(loc_from(arg)?.to_string().into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ciborium::value::Value;

    /// CBOR-encodes a `ciborium` value for use as plugin input.
    fn encode(value: &Value) -> Vec<u8> {
        let mut buf = Vec::new();
        into_writer(value, &mut buf).unwrap();
        buf
    }

    /// Builds the CBOR argument map the Typst side would send.
    fn args(entries: &[(&str, Value)]) -> Vec<u8> {
        let map = Value::Map(
            entries
                .iter()
                .map(|(k, v)| (Value::Text((*k).to_owned()), v.clone()))
                .collect(),
        );
        encode(&map)
    }

    fn author(forename: Option<&str>, surname: Option<&str>) -> Value {
        let mut entries = Vec::new();
        if let Some(name) = forename {
            entries.push((
                Value::Text("forename".to_owned()),
                Value::Text(name.to_owned()),
            ));
        }
        if let Some(name) = surname {
            entries.push((
                Value::Text("surname".to_owned()),
                Value::Text(name.to_owned()),
            ));
        }
        Value::Map(entries)
    }

    /// The value under `key` in a CBOR map, if any.
    fn field(bytes: &[u8], key: &str) -> Option<Value> {
        let value: Value = from_reader(bytes).unwrap();
        let Value::Map(map) = value else {
            panic!("expected a CBOR map, got {value:?}")
        };
        map.into_iter()
            .find(|(k, _)| matches!(k, Value::Text(t) if t == key))
            .map(|(_, v)| v)
    }

    fn text(value: &str) -> Value {
        Value::Text(value.to_owned())
    }

    #[test]
    fn cutter_fmt_matches_known_values() {
        let payload = args(&[
            ("author", author(Some("Charles"), Some("Dickens"))),
            ("title", Value::Text("Oliver Twist".to_owned())),
        ]);
        assert_eq!(cutter_fmt(&payload).unwrap(), b"D548o");
    }

    #[test]
    fn cutter_returns_cbor_parts() {
        let payload = args(&[("author", author(None, Some("Borges")))]);
        let bytes = cutter(&payload).unwrap();
        let get = |key| field(&bytes, key);
        assert_eq!(get("alpha_prefix"), Some(Value::Text("B".to_owned())));
        assert_eq!(get("numeric"), Some(Value::Integer(732.into())));
        assert_eq!(get("entry"), Some(Value::Text("Borg".to_owned())));
        assert_eq!(get("alpha_suffix"), Some(Value::Null));
        // The shared `Detailed` wrapper adds the rendered call number.
        assert_eq!(get("formatted"), Some(Value::Text("B732".to_owned())));
    }

    #[test]
    fn loc_functions_cover_both_shapes() {
        let payload = args(&[("author", author(None, Some("Carter")))]);
        assert_eq!(cutter_loc_fmt(&payload).unwrap(), b"C378");

        let bytes = cutter_loc(&payload).unwrap();
        let get = |key| field(&bytes, key);
        assert_eq!(get("numeric"), Some(Value::Integer(378.into())));
        assert_eq!(get("alpha_prefix"), Some(Value::Text("C".to_owned())));
        // The shared `Detailed` wrapper adds the rendered call number.
        assert_eq!(get("formatted"), Some(Value::Text("C378".to_owned())));
    }

    #[test]
    fn config_overrides_are_honored() {
        let payload = args(&[
            ("author", author(Some("Charles"), Some("Dickens"))),
            ("title", Value::Text("Oliver Twist".to_owned())),
            ("prefix_len", Value::Integer(2.into())),
            ("suffix_len", Value::Integer(3.into())),
        ]);
        assert_eq!(cutter_fmt(&payload).unwrap(), b"Di548oli");

        let loc_payload = args(&[
            ("author", author(None, Some("Macdonald"))),
            ("max_len", Value::Integer(4.into())),
        ]);
        assert_eq!(cutter_loc_fmt(&loc_payload).unwrap(), b"M3336");
    }

    #[test]
    fn blank_author_falls_back_to_title() {
        let payload = args(&[
            ("author", author(None, Some("   "))),
            ("title", Value::Text("Bear".to_owned())),
        ]);
        assert_eq!(cutter_loc_fmt(&payload).unwrap(), b"B437");
    }

    #[test]
    fn blank_field_beside_usable_field_is_ignored() {
        // A blank surname must not poison a usable forename.
        let payload = args(&[("author", author(Some("Prince"), Some("   ")))]);
        assert_eq!(cutter_loc_fmt(&payload).unwrap(), b"P756");
    }

    #[test]
    fn strip_article_files_the_title_without_its_article() {
        let dickens = args(&[
            ("author", author(Some("Charles"), Some("Dickens"))),
            ("title", text("A Christmas Carol")),
            ("lang", text("en")),
            ("strip_article", Value::Bool(true)),
        ]);
        assert_eq!(cutter_fmt(&dickens).unwrap(), b"D548c");

        let bear = args(&[
            ("title", text("The Bear")),
            ("lang", text("EN-us")),
            ("strip_article", Value::Bool(true)),
        ]);
        let detailed = cutter(&bear).unwrap();
        assert_eq!(field(&detailed, "entry"), Some(text("Bear")));
        assert_eq!(field(&detailed, "formatted"), Some(text("B368")));
        assert_eq!(cutter_loc_fmt(&bear).unwrap(), b"B437");
        assert_eq!(
            field(&cutter_loc(&bear).unwrap(), "formatted"),
            Some(text("B437"))
        );
    }

    #[test]
    fn author_names_are_never_stripped() {
        let payload = args(&[
            ("author", author(Some("Jean de"), Some("La Fontaine"))),
            ("title", text("Les Fables")),
            ("lang", text("fr")),
            ("strip_article", Value::Bool(true)),
        ]);
        assert_eq!(cutter_fmt(&payload).unwrap(), b"L166f");
    }

    #[test]
    fn a_language_without_the_switch_changes_nothing() {
        let plain = args(&[("title", text("The Bear"))]);
        for extra in [
            vec![("lang", text("en"))],
            vec![("lang", text("en")), ("strip_article", Value::Bool(false))],
            vec![("lang", text("  ")), ("strip_article", Value::Bool(false))],
        ] {
            let mut entries = vec![("title", text("The Bear"))];
            entries.extend(extra);
            let payload = args(&entries);
            assert_eq!(cutter_fmt(&payload), cutter_fmt(&plain));
            assert_eq!(cutter_loc_fmt(&payload).unwrap(), b"T443");
        }
    }

    #[test]
    fn strip_article_requires_a_language() {
        for lang in [None, Some("   ")] {
            let mut entries = vec![
                ("title", text("The Bear")),
                ("strip_article", Value::Bool(true)),
            ];
            entries.extend(lang.map(|code| ("lang", text(code))));
            let payload = args(&entries);
            assert_eq!(
                cutter_fmt(&payload).unwrap_err(),
                "`strip-article` requires `lang`"
            );
            assert_eq!(
                cutter_loc(&payload).unwrap_err(),
                "`strip-article` requires `lang`"
            );
        }
    }

    #[test]
    fn an_unknown_language_errors_with_the_switch_on_or_off() {
        for switch in [vec![], vec![("strip_article", Value::Bool(true))]] {
            let mut entries = vec![("title", text("The Bear")), ("lang", text("xx"))];
            entries.extend(switch);
            let payload = args(&entries);
            let message = "xx".parse::<Language>().unwrap_err().to_string();
            assert_eq!(cutter_fmt(&payload).unwrap_err(), message);
            assert_eq!(cutter_loc_fmt(&payload).unwrap_err(), message);
        }
    }

    #[test]
    fn language_errors_come_before_input_errors() {
        let unknown = args(&[("lang", text("xx"))]);
        assert!(
            cutter(&unknown)
                .unwrap_err()
                .starts_with("unknown language")
        );
        let missing = args(&[("strip_article", Value::Bool(true))]);
        assert_eq!(
            cutter_loc_fmt(&missing).unwrap_err(),
            "`strip-article` requires `lang`"
        );
    }

    #[test]
    fn mistyped_and_unknown_keys_error() {
        // ciborium words a mistyped bool and a mistyped string differently.
        for (key, value, expected) in [
            ("strip_article", text("yes"), "expected bool"),
            ("strip_article", Value::Integer(1.into()), "expected bool"),
            ("lang", Value::Integer(5.into()), "expected string"),
            ("lang", Value::Bool(true), "expected string"),
        ] {
            let payload = args(&[("title", text("The Bear")), (key, value)]);
            let error = cutter_fmt(&payload).unwrap_err();
            assert!(
                error.starts_with("failed to parse arguments") && error.contains(expected),
                "{key}: {error}"
            );
        }
        let unknown = args(&[
            ("title", text("The Bear")),
            ("strip-article", Value::Bool(true)),
        ]);
        let error = cutter_loc_fmt(&unknown).unwrap_err();
        assert!(error.contains("unknown field `strip-article`"), "{error}");
    }

    #[test]
    fn missing_inputs_error() {
        let empty_map = args(&[]);
        assert!(cutter(&empty_map).unwrap_err().contains("at least one"));

        let not_cbor = b"not cbor at all";
        assert!(
            cutter_fmt(not_cbor)
                .unwrap_err()
                .contains("failed to parse")
        );
    }
}
