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
//! cutterSanborn({ givenName: "Charles", familyName: "Dickens",
//!                 title: "Oliver Twist" });        // "D548o"
//! cutterLoc({ familyName: "Carter" });             // "C378"
//! cutterSanbornDetailed({ familyName: "Borges" }); // { alpha_prefix: "B", … }
//! ```
//!
//! At least one of `givenName`/`familyName`/`title` must be non-blank;
//! blank strings count as absent. Errors are thrown as JavaScript `Error`s.
//!
//! The `*Detailed` variants return plain objects with the number's parts
//! plus the rendered call number under `formatted`. The option structs and
//! the `*_impl` functions compile and are unit-tested on the host, so the
//! full logic path is exercised without a browser.

use csan_core::{
    Attribution, CutterConfig, CutterGenerator, CutterLoc, CutterLocMethod, CutterSanborn,
    CutterSanbornMethod, Detailed,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::{JsError, JsValue, wasm_bindgen};

/// Options accepted by [`cutter_sanborn`] and [`cutter_sanborn_detailed`].
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
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
#[serde(rename_all = "camelCase", default)]
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
    given: Option<&str>,
    family: Option<&str>,
    title: Option<&str>,
) -> Result<Attribution, String> {
    Attribution::from_parts(given, family, title).ok_or_else(|| {
        String::from("provide at least one non-blank input: givenName, familyName, or title")
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

/// Deserializes a JS options object, treating `undefined`/`null` as `{}`.
fn parse_options<T: Default + for<'de> Deserialize<'de>>(options: &JsValue) -> Result<T, JsError> {
    if options.is_undefined() || options.is_null() {
        return Ok(T::default());
    }
    serde_wasm_bindgen::from_value(options.clone()).map_err(|e| JsError::new(&e.to_string()))
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
pub fn cutter_sanborn(options: &JsValue) -> Result<String, JsError> {
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
#[wasm_bindgen(js_name = "cutterSanbornDetailed")]
pub fn cutter_sanborn_detailed(options: &JsValue) -> Result<JsValue, JsError> {
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
pub fn cutter_loc(options: &JsValue) -> Result<String, JsError> {
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
#[wasm_bindgen(js_name = "cutterLocDetailed")]
pub fn cutter_loc_detailed(options: &JsValue) -> Result<JsValue, JsError> {
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
            assert_eq!(csan_core::detailed_json(&cutter).unwrap()["alpha_prefix"], "C");

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
            assert!(err.contains("familyName"), "unhelpful message: {err}");

            // The LoC path shares the same guard; check it independently.
            let loc_options = LocOptions {
                forename: Some("  ".to_owned()),
                ..Default::default()
            };
            let loc_err = loc_impl(&loc_options).unwrap_err();
            assert!(
                loc_err.contains("givenName"),
                "unhelpful message: {loc_err}"
            );
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
            // No work mark without a title. The shared `Detailed` adapter
            // serializes an absent suffix through `serde_json`, so the field
            // is present as JS `null` (not `undefined`).
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
        fn blank_input_throws() {
            let arg = opts(&Options {
                surname: Some("   "),
                ..Default::default()
            });
            assert!(cutter_sanborn(&arg).is_err());
        }
    }
}
