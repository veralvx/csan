#![forbid(unsafe_code)]

//! Typst WASM plugin exposing [`csan`]'s Cutter call number generation.
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
//! `suffix_len`, and `max_len` (see [`CutterConfig`]). At least one of
//! `author`/`title` must be usable. Errors are reported as protocol error
//! strings, which Typst surfaces as diagnostics.
//!
//! On non-wasm targets the same functions compile as ordinary library
//! functions, which is how the unit tests exercise the exact plugin surface.

use ciborium::{de::from_reader, ser::into_writer};
use csan::{
    Attribution, Author, CutterConfig, CutterGenerator, CutterLoc, CutterLocMethod, CutterSanborn,
    CutterSanbornMethod, Detailed,
};
use serde::Deserialize;

#[cfg(target_arch = "wasm32")]
wasm_minimal_protocol::initiate_protocol!();

/// The CBOR argument map shared by every plugin function.
#[derive(Deserialize, Debug)]
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

    /// Resolves the argument map into an [`Attribution`].
    ///
    /// Delegates to [`Attribution::from_parts`], so blank strings are
    /// treated as absent — per field: `{"author": {"surname": " ",
    /// "forename": "Prince"}}` cutters on the given name, and
    /// `{"author": {"surname": " "}, "title": "Dune"}` falls back to a
    /// title-derived cutter instead of erroring.
    fn attribution(self) -> Result<Attribution, String> {
        let author = self.author.as_ref();
        Attribution::from_parts(
            author.and_then(Author::forename),
            author.and_then(Author::surname),
            self.title.as_deref(),
        )
        .ok_or_else(|| {
            String::from("at least one of `author` or `title` must be provided and non-blank")
        })
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
/// name the wrong fields, or fail Cutter generation.
#[cfg_attr(target_arch = "wasm32", wasm_minimal_protocol::wasm_func)]
pub fn cutter(arg: &[u8]) -> Result<Vec<u8>, String> {
    to_cbor(&Detailed::new(&sanborn_from(arg)?))
}

/// Returns the Cutter-Sanborn call number as a UTF-8 string
/// (e.g. `D548o`).
///
/// # Errors
/// Returns a protocol error string when the arguments cannot be decoded,
/// name the wrong fields, or fail Cutter generation.
#[cfg_attr(target_arch = "wasm32", wasm_minimal_protocol::wasm_func)]
pub fn cutter_fmt(arg: &[u8]) -> Result<Vec<u8>, String> {
    Ok(sanborn_from(arg)?.to_string().into_bytes())
}

/// Returns the Library of Congress cutter number as a CBOR map with the
/// keys `alpha_prefix`, `numeric`, and `formatted`.
///
/// # Errors
/// Returns a protocol error string when the arguments cannot be decoded,
/// name the wrong fields, or fail Cutter generation.
#[cfg_attr(target_arch = "wasm32", wasm_minimal_protocol::wasm_func)]
pub fn cutter_loc(arg: &[u8]) -> Result<Vec<u8>, String> {
    to_cbor(&Detailed::new(&loc_from(arg)?))
}

/// Returns the Library of Congress cutter number as a UTF-8 string
/// (e.g. `D535`).
///
/// # Errors
/// Returns a protocol error string when the arguments cannot be decoded,
/// name the wrong fields, or fail Cutter generation.
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

    fn author(given: Option<&str>, family: Option<&str>) -> Value {
        let mut entries = Vec::new();
        if let Some(g) = given {
            entries.push((
                Value::Text("forename".to_owned()),
                Value::Text(g.to_owned()),
            ));
        }
        if let Some(f) = family {
            entries.push((
                Value::Text("surname".to_owned()),
                Value::Text(f.to_owned()),
            ));
        }
        Value::Map(entries)
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
        let value: Value = from_reader(bytes.as_slice()).unwrap();
        let Value::Map(map) = value else {
            panic!("expected a CBOR map, got {value:?}")
        };
        let get = |key: &str| {
            map.iter()
                .find(|(k, _)| matches!(k, Value::Text(t) if t == key))
                .map(|(_, v)| v.clone())
        };
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
        let value: Value = from_reader(bytes.as_slice()).unwrap();
        let Value::Map(map) = value else {
            panic!("expected a CBOR map, got {value:?}")
        };
        let get = |key: &str| {
            map.iter()
                .find(|(k, _)| matches!(k, Value::Text(t) if t == key))
                .map(|(_, v)| v.clone())
        };
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
        // A blank family name must not poison a usable given name.
        let payload = args(&[("author", author(Some("Prince"), Some("   ")))]);
        assert_eq!(cutter_loc_fmt(&payload).unwrap(), b"P756");
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
