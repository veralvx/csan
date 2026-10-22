// The wasm-bindgen macro expansions contain `unsafe`, so
// `#![forbid(unsafe_code)]` (which cannot be overridden) is not usable
// here. `deny` still rejects any hand-written `unsafe` in this crate
// while allowing the macro-generated blocks through.
#![deny(unsafe_code)]

//! JavaScript bindings for [`csan_core`]'s Cutter call number generation,
//! built with [`wasm-bindgen`](https://github.com/wasm-bindgen/wasm-bindgen).
//!
//! Compiled to `wasm32-unknown-unknown` (typically via `wasm-pack`), this
//! crate exports four functions, each taking a single options object with
//! camelCase keys:
//!
//! ```js
//! import { cutterSanborn, cutterLoc } from "csan-web";
//!
//! cutterSanborn({ forename: "Charles", surname: "Dickens",
//!                 title: "Oliver Twist" });     // "D548o"
//! cutterLoc({ surname: "Carter" });             // "C378"
//! cutterSanbornDetailed({ surname: "Borges" }); // { alpha_prefix: "B", … }
//! ```
//!
//! At least one of `forename`/`surname`/`title` must be non-blank;
//! blank strings count as absent. Unknown option keys are rejected rather
//! than ignored, so a misspelled key cannot silently change the result.
//! Errors are thrown as JavaScript `Error`s.
//!
//! The `*Detailed` variants return plain objects with the number's parts
//! plus the rendered call number under `formatted`. The option structs and
//! the `*_impl` functions compile and are unit-tested on the host, so the
//! full logic path is exercised without a browser.

use csan_core::{
    Attribution, CutterConfig, CutterGenerator, CutterLoc, CutterLocMethod, CutterSanborn,
    CutterSanbornMethod, Detailed,
};
use serde::de::value::MapAccessDeserializer;
use serde::de::{Deserializer, MapAccess, Visitor};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::marker::PhantomData;
use wasm_bindgen::prelude::{JsError, JsValue, wasm_bindgen};

// TypeScript declarations for the options and detailed results; the
// exports below reference them through `unchecked_*_type`, so the generated
// `.d.ts` is typed instead of `any`.
#[wasm_bindgen(typescript_custom_section)]
const TS_TYPES: &str = r#"
/** Options for `cutterSanborn` and `cutterSanbornDetailed`. */
export interface SanbornOptions {
  /** The author's given (first) name. */
  forename?: string | null;
  /** The author's family (last) name. */
  surname?: string | null;
  /** The work's title (work mark source, or title-derived cutters). */
  title?: string | null;
  /** Letters taken from the name for the call number prefix (default 1). */
  prefixLen?: number | null;
  /** Letters taken from the title for the work mark (default 1). */
  suffixLen?: number | null;
}

/** Options for `cutterLoc` and `cutterLocDetailed`. */
export interface LocOptions {
  /** The author's given (first) name. */
  forename?: string | null;
  /** The author's family (last) name. */
  surname?: string | null;
  /** The work's title (used when no author is given). */
  title?: string | null;
  /** Digits in the number, clamped to 4 (default 3). */
  maxLen?: number | null;
}

/** The parts of a Cutter-Sanborn number. */
export interface CutterSanbornDetailed {
  alpha_prefix: string;
  numeric: number;
  /** The work mark; absent when no title was given. */
  alpha_suffix?: string;
  /** The Cutter-Sanborn table key that matched. */
  entry: string;
  formatted: string;
}

/** The parts of a Library of Congress cutter number. */
export interface CutterLocDetailed {
  alpha_prefix: string;
  numeric: number;
  formatted: string;
}
"#;

/// Options accepted by [`cutter_sanborn`] and [`cutter_sanborn_detailed`].
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct SanbornOptions {
    /// The author's given (first) name.
    pub forename: Option<String>,
    /// The author's family (last) name.
    pub surname: Option<String>,
    /// The work's title (work mark source, or title-derived cutters).
    pub title: Option<String>,
    /// Letters taken from the name for the call number prefix (default 1).
    pub prefix_len: Option<usize>,
    /// Letters taken from the title for the work mark (default 1).
    pub suffix_len: Option<usize>,
}

/// Options accepted by [`cutter_loc`] and [`cutter_loc_detailed`].
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct LocOptions {
    /// The author's given (first) name.
    pub forename: Option<String>,
    /// The author's family (last) name.
    pub surname: Option<String>,
    /// The work's title (used when no author is given).
    pub title: Option<String>,
    /// Digits in the number, clamped to 4 by the library (default 3).
    pub max_len: Option<usize>,
}

/// Builds the [`Attribution`] for a set of optional inputs, with a
/// JS-friendly error message.
fn attribution(
    forename: Option<&str>,
    surname: Option<&str>,
    title: Option<&str>,
) -> Result<Attribution, String> {
    Attribution::from_parts(forename, surname, title).ok_or_else(|| {
        String::from("provide at least one non-blank input: forename, surname, or title")
    })
}

/// Host-testable core of [`cutter_sanborn`] / [`cutter_sanborn_detailed`].
///
/// # Errors
/// Returns a message when no usable input was provided or generation fails.
pub fn sanborn_impl(options: &SanbornOptions) -> Result<CutterSanborn, String> {
    let entry = attribution(
        options.forename.as_deref(),
        options.surname.as_deref(),
        options.title.as_deref(),
    )?;
    let config = CutterConfig::from_overrides(options.prefix_len, options.suffix_len, None);
    CutterSanbornMethod::generate(&entry, config).map_err(|e| e.to_string())
}

/// Host-testable core of [`cutter_loc`] / [`cutter_loc_detailed`].
///
/// # Errors
/// Returns a message when no usable input was provided or generation fails.
pub fn loc_impl(options: &LocOptions) -> Result<CutterLoc, String> {
    let entry = attribution(
        options.forename.as_deref(),
        options.surname.as_deref(),
        options.title.as_deref(),
    )?;
    let config = CutterConfig::from_overrides(None, None, options.max_len);
    CutterLocMethod::generate(&entry, config).map_err(|e| e.to_string())
}

/// Deserializes `T` through the deserializer's *map* path.
///
/// serde-wasm-bindgen's struct path only looks up the field names `T`
/// declares, so keys `T` does not know never reach it and
/// `deny_unknown_fields` cannot fire. Its map path enumerates every own key
/// of the object; replaying those entries into `T` lets the derived
/// visitor reject a misspelled key instead of silently ignoring it.
struct AllKeys<T>(T);

impl<'de, T: Deserialize<'de>> Deserialize<'de> for AllKeys<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct EntriesVisitor<T>(PhantomData<T>);

        impl<'de, T: Deserialize<'de>> Visitor<'de> for EntriesVisitor<T> {
            type Value = T;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("an options object")
            }

            fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<T, A::Error> {
                T::deserialize(MapAccessDeserializer::new(map))
            }
        }

        deserializer
            .deserialize_map(EntriesVisitor(PhantomData))
            .map(AllKeys)
    }
}

/// Deserializes a JS options object, treating `undefined`/`null` as `{}`.
///
/// Only the object's own properties are read (inherited ones are not).
fn parse_options<T: Default + for<'de> Deserialize<'de>>(options: &JsValue) -> Result<T, JsError> {
    if options.is_undefined() || options.is_null() {
        return Ok(T::default());
    }
    // The map path would otherwise fail on a primitive with an opaque
    // `Reflect.get` TypeError.
    if !options.is_object() {
        return Err(JsError::new("options must be an object"));
    }
    serde_wasm_bindgen::from_value::<AllKeys<T>>(options.clone())
        .map(|AllKeys(parsed)| parsed)
        .map_err(|e| JsError::new(&e.to_string()))
}

/// Serializes a value into a plain JS object.
///
/// The [`Detailed`](csan_core::Detailed) adapter serializes through serde's map
/// interface; serde-wasm-bindgen would otherwise render that as a JS `Map`,
/// so `serialize_maps_as_objects` keeps the output a plain object with the
/// fields as properties (what JS callers expect).
fn to_js(value: &impl Serialize) -> Result<JsValue, JsError> {
    let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
    value
        .serialize(&serializer)
        .map_err(|e| JsError::new(&e.to_string()))
}

/// Returns the Cutter-Sanborn call number as a string (e.g. `"D548o"`).
///
/// # Errors
/// Throws when the options are malformed, no usable input was provided, or
/// generation fails.
#[wasm_bindgen(js_name = "cutterSanborn")]
pub fn cutter_sanborn(
    #[wasm_bindgen(unchecked_param_type = "SanbornOptions | null | undefined")] options: &JsValue,
) -> Result<String, JsError> {
    let parsed: SanbornOptions = parse_options(options)?;
    sanborn_impl(&parsed)
        .map(|c| c.to_string())
        .map_err(|e| JsError::new(&e))
}

/// Returns the parts of the Cutter-Sanborn number as an object
/// (`{ alpha_prefix, numeric, alpha_suffix, entry, formatted }`).
///
/// # Errors
/// Throws when the options are malformed, no usable input was provided, or
/// generation fails.
#[wasm_bindgen(
    js_name = "cutterSanbornDetailed",
    unchecked_return_type = "CutterSanbornDetailed"
)]
pub fn cutter_sanborn_detailed(
    #[wasm_bindgen(unchecked_param_type = "SanbornOptions | null | undefined")] options: &JsValue,
) -> Result<JsValue, JsError> {
    let parsed: SanbornOptions = parse_options(options)?;
    let cutter = sanborn_impl(&parsed).map_err(|e| JsError::new(&e))?;
    to_js(&Detailed::new(&cutter))
}

/// Returns the Library of Congress cutter number as a string
/// (e.g. `"C378"`).
///
/// # Errors
/// Throws when the options are malformed, no usable input was provided, or
/// generation fails.
#[wasm_bindgen(js_name = "cutterLoc")]
pub fn cutter_loc(
    #[wasm_bindgen(unchecked_param_type = "LocOptions | null | undefined")] options: &JsValue,
) -> Result<String, JsError> {
    let parsed: LocOptions = parse_options(options)?;
    loc_impl(&parsed)
        .map(|c| c.to_string())
        .map_err(|e| JsError::new(&e))
}

/// Returns the parts of the Library of Congress number as an object
/// (`{ alpha_prefix, numeric, formatted }`).
///
/// # Errors
/// Throws when the options are malformed, no usable input was provided, or
/// generation fails.
#[wasm_bindgen(
    js_name = "cutterLocDetailed",
    unchecked_return_type = "CutterLocDetailed"
)]
pub fn cutter_loc_detailed(
    #[wasm_bindgen(unchecked_param_type = "LocOptions | null | undefined")] options: &JsValue,
) -> Result<JsValue, JsError> {
    let parsed: LocOptions = parse_options(options)?;
    let cutter = loc_impl(&parsed).map_err(|e| JsError::new(&e))?;
    to_js(&Detailed::new(&cutter))
}
// The crate's tests split by target. `tests::host` covers the pure-Rust
// cores and conversions on the host; `tests::wasm` drives the exported
// `JsValue` surface under a wasm-bindgen-test runner. Each inner module
// carries its target guard as an outer attribute, and the enclosing
// `#[cfg(test)]` keeps the `tests_outside_test_module` lint satisfied.
#[cfg(test)]
mod tests {
    #[cfg(not(target_arch = "wasm32"))]
    mod host {
        // `js-sys` and `wasm-bindgen-test` are used only by the wasm-target
        // `wasm` module; reference them here so the per-target
        // `unused_crate_dependencies` lint stays quiet on host test builds.
        use {js_sys as _, wasm_bindgen_test as _};

        use crate::*;

        #[test]
        fn sanborn_impl_matches_known_values() {
            let options = SanbornOptions {
                forename: Some("Charles".to_owned()),
                surname: Some("Dickens".to_owned()),
                title: Some("Oliver Twist".to_owned()),
                ..Default::default()
            };
            let cutter = sanborn_impl(&options).unwrap();
            assert_eq!(cutter.to_string(), "D548o");

            let parts = csan_core::detailed_json(&cutter).unwrap();
            assert_eq!(parts["alpha_prefix"], "D");
            assert_eq!(parts["numeric"], 548);
            assert_eq!(parts["alpha_suffix"], "o");
            assert_eq!(parts["formatted"], "D548o");
        }

        #[test]
        fn sanborn_config_overrides_apply() {
            let options = SanbornOptions {
                forename: Some("Charles".to_owned()),
                surname: Some("Dickens".to_owned()),
                title: Some("Oliver Twist".to_owned()),
                prefix_len: Some(2),
                suffix_len: Some(3),
            };
            assert_eq!(sanborn_impl(&options).unwrap().to_string(), "Di548oli");
        }

        #[test]
        fn loc_impl_matches_known_values() {
            let options = LocOptions {
                surname: Some("Carter".to_owned()),
                ..Default::default()
            };
            let cutter = loc_impl(&options).unwrap();
            assert_eq!(cutter.to_string(), "C378");
            assert_eq!(
                csan_core::detailed_json(&cutter).unwrap()["alpha_prefix"],
                "C"
            );

            let clamped = LocOptions {
                surname: Some("Macdonald".to_owned()),
                max_len: Some(64),
                ..Default::default()
            };
            assert_eq!(loc_impl(&clamped).unwrap().to_string(), "M3336");
        }

        #[test]
        fn blank_inputs_error_with_js_field_names() {
            let options = SanbornOptions {
                surname: Some("   ".to_owned()),
                ..Default::default()
            };
            let err = sanborn_impl(&options).unwrap_err();
            assert!(err.contains("surname"), "unhelpful message: {err}");

            // The LoC path shares the same guard; check it independently.
            let loc_options = LocOptions {
                forename: Some("  ".to_owned()),
                ..Default::default()
            };
            let loc_err = loc_impl(&loc_options).unwrap_err();
            assert!(loc_err.contains("forename"), "unhelpful message: {loc_err}");
        }

        #[test]
        fn loc_parts_expose_every_field() {
            let cutter = loc_impl(&LocOptions {
                surname: Some("Carter".to_owned()),
                ..Default::default()
            })
            .unwrap();
            let parts = csan_core::detailed_json(&cutter).unwrap();
            assert_eq!(parts["alpha_prefix"], "C");
            assert_eq!(parts["numeric"], 378);
            assert_eq!(parts["formatted"], "C378");
        }

        #[test]
        fn unknown_option_keys_are_rejected() {
            // A misspelled or stale key (here the pre-rename `givenName`)
            // must fail loudly: ignoring it would silently cutter on the
            // remaining fields (e.g. the title) and return a wrong number.
            use serde::de::value::{Error, MapDeserializer};
            let fields = [("givenName", "Charles"), ("title", "Oliver Twist")];
            let sanborn = AllKeys::<SanbornOptions>::deserialize(MapDeserializer::<_, Error>::new(
                fields.into_iter(),
            ));
            let err = sanborn.err().unwrap().to_string();
            assert!(err.contains("givenName"), "unhelpful message: {err}");

            // `prefixLen` is a Cutter-Sanborn knob; the LoC options reject it.
            let loc = AllKeys::<LocOptions>::deserialize(MapDeserializer::<_, Error>::new(
                [("prefixLen", "2")].into_iter(),
            ));
            let loc_err = loc.err().unwrap().to_string();
            assert!(
                loc_err.contains("prefixLen"),
                "unhelpful message: {loc_err}"
            );
        }

        #[test]
        fn title_only_uses_the_numeral_rule() {
            let options = LocOptions {
                title: Some("100 best recipes".to_owned()),
                ..Default::default()
            };
            assert_eq!(loc_impl(&options).unwrap().to_string(), "A153");
        }
    }

    #[cfg(target_arch = "wasm32")]
    mod wasm {
        // `js-sys` and `wasm-bindgen-test` are only reachable here, on the
        // wasm target; on the host `cargo test` build they would otherwise
        // trip `unused_crate_dependencies`, so the whole module is gated.
        use crate::*;
        use serde::Serialize;
        use wasm_bindgen::JsValue;
        use wasm_bindgen_test::wasm_bindgen_test;

        /// A camelCase options object, serialized to a `JsValue` exactly as a
        /// JavaScript caller would pass one.
        #[derive(Default, Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Options {
            #[serde(skip_serializing_if = "Option::is_none")]
            forename: Option<&'static str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            surname: Option<&'static str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            title: Option<&'static str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            prefix_len: Option<usize>,
            #[serde(skip_serializing_if = "Option::is_none")]
            suffix_len: Option<usize>,
            #[serde(skip_serializing_if = "Option::is_none")]
            max_len: Option<usize>,
        }

        /// Serializes an options struct into the `JsValue` the exports expect.
        fn opts(options: &Options) -> JsValue {
            serde_wasm_bindgen::to_value(options).unwrap()
        }

        /// Reads a string field from a returned detail object.
        fn field_str(value: &JsValue, key: &str) -> Option<String> {
            let got = js_sys::Reflect::get(value, &JsValue::from_str(key)).unwrap();
            got.as_string()
        }

        /// Reads a numeric field from a returned detail object.
        fn field_f64(value: &JsValue, key: &str) -> Option<f64> {
            let got = js_sys::Reflect::get(value, &JsValue::from_str(key)).unwrap();
            got.as_f64()
        }

        #[wasm_bindgen_test]
        fn cutter_sanborn_returns_the_call_number() {
            let arg = opts(&Options {
                forename: Some("Charles"),
                surname: Some("Dickens"),
                title: Some("Oliver Twist"),
                ..Default::default()
            });
            assert_eq!(cutter_sanborn(&arg).unwrap(), "D548o");
        }

        #[wasm_bindgen_test]
        fn cutter_sanborn_honors_overrides() {
            let arg = opts(&Options {
                surname: Some("Dickens"),
                title: Some("Oliver Twist"),
                prefix_len: Some(2),
                suffix_len: Some(3),
                ..Default::default()
            });
            assert_eq!(cutter_sanborn(&arg).unwrap(), "Di548oli");
        }

        #[wasm_bindgen_test]
        fn cutter_sanborn_detailed_returns_the_parts() {
            let arg = opts(&Options {
                surname: Some("Borges"),
                ..Default::default()
            });
            let value = cutter_sanborn_detailed(&arg).unwrap();
            assert_eq!(field_str(&value, "alpha_prefix").as_deref(), Some("B"));
            assert_eq!(field_f64(&value, "numeric"), Some(732.0));
            assert_eq!(field_str(&value, "entry").as_deref(), Some("Borg"));
            assert_eq!(field_str(&value, "formatted").as_deref(), Some("B732"));
            // No work mark without a title. serde-wasm-bindgen serializes an
            // absent suffix (JSON null, from the shared `Detailed` adapter) as
            // JS `undefined`, so the value round-trips as `undefined`.
            let suffix = js_sys::Reflect::get(&value, &JsValue::from_str("alpha_suffix")).unwrap();
            assert!(suffix.is_undefined());
        }

        #[wasm_bindgen_test]
        fn cutter_loc_returns_the_call_number() {
            let arg = opts(&Options {
                surname: Some("Carter"),
                ..Default::default()
            });
            assert_eq!(cutter_loc(&arg).unwrap(), "C378");

            let clamped = opts(&Options {
                surname: Some("Macdonald"),
                max_len: Some(64),
                ..Default::default()
            });
            assert_eq!(cutter_loc(&clamped).unwrap(), "M3336");
        }

        #[wasm_bindgen_test]
        fn cutter_loc_detailed_returns_the_parts() {
            let arg = opts(&Options {
                title: Some("100 best recipes"),
                ..Default::default()
            });
            let value = cutter_loc_detailed(&arg).unwrap();
            assert_eq!(field_str(&value, "alpha_prefix").as_deref(), Some("A"));
            assert_eq!(field_f64(&value, "numeric"), Some(153.0));
            assert_eq!(field_str(&value, "formatted").as_deref(), Some("A153"));
        }

        #[wasm_bindgen_test]
        fn missing_input_throws() {
            assert!(cutter_sanborn(&JsValue::UNDEFINED).is_err());
            assert!(cutter_loc(&opts(&Options::default())).is_err());
        }

        #[wasm_bindgen_test]
        fn non_object_options_throw() {
            assert!(cutter_loc(&JsValue::from_str("Carter")).is_err());
            assert!(cutter_sanborn(&JsValue::from_f64(1.0)).is_err());
        }

        #[wasm_bindgen_test]
        fn unknown_option_keys_throw() {
            let arg = js_sys::Object::new();
            js_sys::Reflect::set(&arg, &"givenName".into(), &"Charles".into()).unwrap();
            js_sys::Reflect::set(&arg, &"title".into(), &"Oliver Twist".into()).unwrap();
            assert!(cutter_sanborn(&arg).is_err());
            assert!(cutter_loc(&arg).is_err());
        }

        #[wasm_bindgen_test]
        fn blank_input_throws() {
            let arg = opts(&Options {
                surname: Some("   "),
                ..Default::default()
            });
            assert!(cutter_sanborn(&arg).is_err());
        }
    }
}
