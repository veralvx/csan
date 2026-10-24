// The magnus macro expansions contain `unsafe`, so
// `#![forbid(unsafe_code)]` (which cannot be overridden) is not usable
// here. `deny` still rejects any hand-written `unsafe` in this crate
// while allowing the macro-generated blocks through.
#![deny(unsafe_code)]

//! The `csan` gem's native extension, `Csan::Native`.
//!
//! Exposes [`csan_core`]'s Cutter call number generation to Ruby, plus the
//! `run_cli` entry point behind the gem's `csan` executable (the same
//! `clap` interface as the standalone binary, provided by [`csan`]).
//! The user-facing API (keyword arguments, documentation) lives in
//! `lib/csan.rb`, which calls these with positional arguments.

use core::fmt;
use core::iter;
use csan_core::{
    Attribution, CutterConfig, CutterGenerator as _, CutterLoc, CutterLocMethod, CutterSanborn,
    CutterSanbornMethod, detailed_json,
};
use magnus::value::ReprValue as _;
use magnus::{Error, IntoValue as _, RHash, Ruby, Value};
use serde_json::Value as Json;

/// Builds the [`Attribution`] for a set of optional inputs, or Ruby's
/// `ArgumentError`.
fn attribution(
    ruby: &Ruby,
    forename: Option<&str>,
    surname: Option<&str>,
    title: Option<&str>,
) -> Result<Attribution, Error> {
    Attribution::from_parts(forename, surname, title).ok_or_else(|| {
        Error::new(
            ruby.exception_arg_error(),
            "provide at least one non-blank input: forename, surname, or title",
        )
    })
}

/// Maps a failure to Ruby's `ArgumentError`, prefixed with `context`.
fn argument_error(ruby: &Ruby, context: &str, error: &impl fmt::Display) -> Error {
    Error::new(ruby.exception_arg_error(), format!("{context}: {error}"))
}

/// Generates the Cutter-Sanborn value shared by both `cutter_sanborn`
/// variants.
fn sanborn(
    ruby: &Ruby,
    forename: Option<&str>,
    surname: Option<&str>,
    title: Option<&str>,
    prefix_len: Option<usize>,
    suffix_len: Option<usize>,
) -> Result<CutterSanborn, Error> {
    let entry = attribution(ruby, forename, surname, title)?;
    let config = CutterConfig::from_overrides(prefix_len, suffix_len, None);
    CutterSanbornMethod::generate(&entry, config)
        .map_err(|error| argument_error(ruby, "cutter generation failed", &error))
}

/// Generates the `LoC` value shared by both `cutter_loc` variants.
fn loc(
    ruby: &Ruby,
    forename: Option<&str>,
    surname: Option<&str>,
    title: Option<&str>,
    max_len: Option<usize>,
) -> Result<CutterLoc, Error> {
    let entry = attribution(ruby, forename, surname, title)?;
    let config = CutterConfig::from_overrides(None, None, max_len);
    CutterLocMethod::generate(&entry, config)
        .map_err(|error| argument_error(ruby, "cutter generation failed", &error))
}

/// Converts a JSON value produced by [`detailed_json`] into the equivalent
/// Ruby object: objects become hashes with symbol keys.
///
/// The detailed objects only ever contain strings, integers, nested objects,
/// and JSON null (for an absent work mark), so those are the cases that
/// matter; other shapes are mapped on a best-effort basis to keep the
/// function total.
fn json_to_ruby(ruby: &Ruby, value: &Json) -> Result<Value, Error> {
    Ok(match value {
        Json::Null => ruby.qnil().as_value(),
        Json::Bool(boolean) => boolean.into_value_with(ruby),
        Json::String(string) => string.as_str().into_value_with(ruby),
        Json::Number(number) => {
            if let Some(unsigned) = number.as_u64() {
                unsigned.into_value_with(ruby)
            } else if let Some(signed) = number.as_i64() {
                signed.into_value_with(ruby)
            } else if let Some(float) = number.as_f64() {
                float.into_value_with(ruby)
            } else {
                // No numeric representation fits; fall back to the text form.
                number.to_string().into_value_with(ruby)
            }
        }
        Json::Array(items) => {
            let array = ruby.ary_new_capa(items.len());
            for item in items {
                array.push(json_to_ruby(ruby, item)?)?;
            }
            array.as_value()
        }
        Json::Object(_) => detailed_hash(ruby, value)?.as_value(),
    })
}

/// Builds the detailed hash from a cutter's [`detailed_json`] object, so
/// every frontend agrees on the shape.
///
/// The value is always a JSON object (the cutter models are structs); a
/// non-object is treated as a programming error and surfaced as
/// `ArgumentError`.
fn detailed_hash(ruby: &Ruby, value: &Json) -> Result<RHash, Error> {
    let Json::Object(fields) = value else {
        return Err(Error::new(
            ruby.exception_arg_error(),
            "detailed cutter value did not serialize as an object",
        ));
    };
    let hash = ruby.hash_new();
    for (key, field) in fields {
        hash.aset(ruby.to_symbol(key), json_to_ruby(ruby, field)?)?;
    }
    Ok(hash)
}

/// The Cutter-Sanborn call number as a string (e.g. `"D548o"`).
#[expect(
    clippy::needless_pass_by_value,
    reason = "magnus passes Ruby's arguments as owned values"
)]
fn cutter_sanborn(
    ruby: &Ruby,
    forename: Option<String>,
    surname: Option<String>,
    title: Option<String>,
    prefix_len: Option<usize>,
    suffix_len: Option<usize>,
) -> Result<String, Error> {
    let cutter = sanborn(
        ruby,
        forename.as_deref(),
        surname.as_deref(),
        title.as_deref(),
        prefix_len,
        suffix_len,
    )?;
    Ok(cutter.to_string())
}

/// The Cutter-Sanborn number's parts: `alpha_prefix`, `numeric`,
/// `alpha_suffix` (nil without a work mark), `entry`, and `formatted`.
#[expect(
    clippy::needless_pass_by_value,
    reason = "magnus passes Ruby's arguments as owned values"
)]
fn cutter_sanborn_detailed(
    ruby: &Ruby,
    forename: Option<String>,
    surname: Option<String>,
    title: Option<String>,
    prefix_len: Option<usize>,
    suffix_len: Option<usize>,
) -> Result<RHash, Error> {
    let cutter = sanborn(
        ruby,
        forename.as_deref(),
        surname.as_deref(),
        title.as_deref(),
        prefix_len,
        suffix_len,
    )?;
    let value = detailed_json(&cutter)
        .map_err(|error| argument_error(ruby, "detailed cutter value failed", &error))?;
    detailed_hash(ruby, &value)
}

/// The Library of Congress cutter number as a string (e.g. `"C378"`).
#[expect(
    clippy::needless_pass_by_value,
    reason = "magnus passes Ruby's arguments as owned values"
)]
fn cutter_loc(
    ruby: &Ruby,
    forename: Option<String>,
    surname: Option<String>,
    title: Option<String>,
    max_len: Option<usize>,
) -> Result<String, Error> {
    let cutter = loc(
        ruby,
        forename.as_deref(),
        surname.as_deref(),
        title.as_deref(),
        max_len,
    )?;
    Ok(cutter.to_string())
}

/// The Library of Congress number's parts: `alpha_prefix`, `numeric`, and
/// `formatted`.
#[expect(
    clippy::needless_pass_by_value,
    reason = "magnus passes Ruby's arguments as owned values"
)]
fn cutter_loc_detailed(
    ruby: &Ruby,
    forename: Option<String>,
    surname: Option<String>,
    title: Option<String>,
    max_len: Option<usize>,
) -> Result<RHash, Error> {
    let cutter = loc(
        ruby,
        forename.as_deref(),
        surname.as_deref(),
        title.as_deref(),
        max_len,
    )?;
    let value = detailed_json(&cutter)
        .map_err(|error| argument_error(ruby, "detailed cutter value failed", &error))?;
    detailed_hash(ruby, &value)
}

/// Runs the `csan` command-line interface on `args` (without the program
/// name) and returns the process exit code: the engine behind the gem's
/// `csan` executable.
fn run_cli(args: Vec<String>) -> u8 {
    csan::run_from_args(iter::once(String::from("csan")).chain(args))
}

/// The extension's entry point, `Init_csan_rb`, which Ruby calls when
/// `lib/csan.rb` requires the extension. In a private module:
/// `#[magnus::init]` exports it whatever its visibility, and it is no public
/// API to document.
mod entry {
    use magnus::{Error, Ruby, function, prelude::*};

    use super::{
        cutter_loc, cutter_loc_detailed, cutter_sanborn, cutter_sanborn_detailed, run_cli,
    };

    /// Defines `Csan::Native` and its module functions, and
    /// `Csan::Native::VERSION`, the crate's version.
    #[magnus::init]
    fn init(ruby: &Ruby) -> Result<(), Error> {
        let native = ruby.define_module("Csan")?.define_module("Native")?;
        native.define_module_function("cutter_sanborn", function!(cutter_sanborn, 5))?;
        native.define_module_function(
            "cutter_sanborn_detailed",
            function!(cutter_sanborn_detailed, 5),
        )?;
        native.define_module_function("cutter_loc", function!(cutter_loc, 4))?;
        native.define_module_function("cutter_loc_detailed", function!(cutter_loc_detailed, 4))?;
        native.define_module_function("run_cli", function!(run_cli, 1))?;
        native.const_set("VERSION", env!("CARGO_PKG_VERSION"))?;
        Ok(())
    }
}
