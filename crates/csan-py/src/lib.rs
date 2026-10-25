// The pyo3 macro expansions contain `unsafe`, so
// `#![forbid(unsafe_code)]` (which cannot be overridden) is not usable
// here. `deny` still rejects any hand-written `unsafe` in this crate
// while allowing the macro-generated blocks through.
#![deny(unsafe_code)]

//! The `csan._csan` Python extension module.
//!
//! Exposes [`csan_core`]'s Cutter call number generation to Python, plus the
//! `run_cli` entry point that powers the `csan` console script (the same
//! `clap` interface as the standalone binary, provided by [`csan`]).
//! The user-facing package (docstrings, `main()`, re-exports) lives in
//! `python/csan/__init__.py`.

use csan_core::{
    Attribution, CutterConfig, CutterGenerator, CutterLoc, CutterLocMethod, CutterSanborn,
    CutterSanbornMethod, Language, MissingLanguage, detailed_json,
};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use serde_json::Value;
use std::fmt;
use std::iter;

/// The language the title is filed in, from the `lang` and
/// `strip_article` arguments: `None` when the switch is off.
///
/// `lang` is parsed even with the switch off, so a bad code is always an
/// error; a blank `lang` counts as absent. The exported functions call it
/// before checking their inputs, so a language error comes first, as in
/// the CLI.
fn article_language(lang: Option<&str>, strip_article: bool) -> PyResult<Option<Language>> {
    let language =
        Language::from_optional(lang).map_err(|e| PyValueError::new_err(e.to_string()))?;
    Language::for_rule(language, strip_article)
        .map_err(|MissingLanguage| PyValueError::new_err("strip_article requires lang"))
}

/// Builds the [`Attribution`] for a set of optional inputs, with its title
/// filed in `article_language`, and a Python-friendly error.
fn attribution(
    forename: Option<&str>,
    surname: Option<&str>,
    title: Option<&str>,
    article_language: Option<Language>,
) -> PyResult<Attribution> {
    let entry = Attribution::from_parts(forename, surname, title).ok_or_else(|| {
        PyValueError::new_err("provide at least one non-blank input: forename, surname, or title")
    })?;
    Ok(entry.with_article_language(article_language))
}

/// Maps a generation failure to a Python `ValueError`.
fn generation_error(error: &impl fmt::Display) -> PyErr {
    PyValueError::new_err(format!("cutter generation failed: {error}"))
}

/// Generates the Cutter-Sanborn value shared by both `cutter_sanborn`
/// variants.
fn sanborn(
    forename: Option<&str>,
    surname: Option<&str>,
    title: Option<&str>,
    prefix_len: Option<usize>,
    suffix_len: Option<usize>,
    article_language: Option<Language>,
) -> PyResult<CutterSanborn> {
    let entry = attribution(forename, surname, title, article_language)?;
    let config = CutterConfig::from_overrides(prefix_len, suffix_len, None);
    CutterSanbornMethod::generate(&entry, config).map_err(|e| generation_error(&e))
}

/// Generates the `LoC` value shared by both `cutter_loc` variants.
fn loc(
    forename: Option<&str>,
    surname: Option<&str>,
    title: Option<&str>,
    max_len: Option<usize>,
    article_language: Option<Language>,
) -> PyResult<CutterLoc> {
    let entry = attribution(forename, surname, title, article_language)?;
    let config = CutterConfig::from_overrides(None, None, max_len);
    CutterLocMethod::generate(&entry, config).map_err(|e| generation_error(&e))
}

/// Return the Cutter-Sanborn call number as a string (e.g. ``"D548o"``).
///
/// At least one of ``forename``, ``surname``, or ``title`` must be
/// non-blank; blank strings count as absent. ``prefix_len`` and
/// ``suffix_len`` default to 1.
///
/// With ``strip_article=True``, the title is filed without its initial
/// article in the language ``lang`` names: an ISO 639 code, a BCP 47 tag or
/// a POSIX locale (``"en"``, ``"pt-BR"``), case-insensitive; ``csan --help``
/// lists them all. ``lang`` alone changes nothing, and a blank ``lang``
/// counts as absent. Author names are never stripped.
///
/// Raises ``ValueError`` on unusable input, an unknown ``lang`` (even with
/// ``strip_article`` off), or ``strip_article`` without ``lang``.
#[pyfunction]
#[pyo3(signature = (forename=None, surname=None, title=None, prefix_len=None, suffix_len=None, *, lang=None, strip_article=false))]
fn cutter_sanborn(
    forename: Option<&str>,
    surname: Option<&str>,
    title: Option<&str>,
    prefix_len: Option<usize>,
    suffix_len: Option<usize>,
    lang: Option<&str>,
    strip_article: bool,
) -> PyResult<String> {
    let language = article_language(lang, strip_article)?;
    Ok(sanborn(forename, surname, title, prefix_len, suffix_len, language)?.to_string())
}

/// Converts a JSON value produced by [`detailed_json`] into the equivalent
/// Python object.
///
/// The detailed objects only ever contain strings, integers, nested objects,
/// and JSON null (for an absent work mark), so those are the cases that
/// matter; other shapes are mapped on a best-effort basis to keep the
/// function total.
fn json_to_py<'py>(py: Python<'py>, value: &Value) -> PyResult<Bound<'py, PyAny>> {
    match value {
        Value::Null => Ok(py.None().into_bound(py)),
        Value::Bool(b) => Ok(b.into_pyobject(py)?.to_owned().into_any()),
        Value::String(s) => Ok(s.into_pyobject(py)?.into_any()),
        Value::Number(n) => {
            if let Some(unsigned) = n.as_u64() {
                Ok(unsigned.into_pyobject(py)?.into_any())
            } else if let Some(signed) = n.as_i64() {
                Ok(signed.into_pyobject(py)?.into_any())
            } else if let Some(float) = n.as_f64() {
                Ok(float.into_pyobject(py)?.into_any())
            } else {
                // No numeric representation fits; fall back to the text form.
                Ok(n.to_string().into_pyobject(py)?.into_any())
            }
        }
        Value::Array(items) => {
            let list = PyList::empty(py);
            for item in items {
                list.append(json_to_py(py, item)?)?;
            }
            Ok(list.into_any())
        }
        Value::Object(fields) => {
            let dict = PyDict::new(py);
            for (key, field_value) in fields {
                dict.set_item(key, json_to_py(py, field_value)?)?;
            }
            Ok(dict.into_any())
        }
    }
}

/// Builds the detailed dict from a cutter's already-computed
/// [`detailed_json`] object, so every frontend agrees on the shape.
///
/// The value is always a JSON object (the cutter models are structs); a
/// non-object is treated as a programming error and surfaced as
/// ``ValueError``.
fn detailed_dict<'py>(py: Python<'py>, value: &Value) -> PyResult<Bound<'py, PyDict>> {
    let Value::Object(fields) = value else {
        return Err(PyValueError::new_err(
            "detailed cutter value did not serialize as an object",
        ));
    };
    let dict = PyDict::new(py);
    for (key, field_value) in fields {
        dict.set_item(key, json_to_py(py, field_value)?)?;
    }
    Ok(dict)
}

/// Return the parts of the Cutter-Sanborn number as a dict with the keys
/// ``alpha_prefix``, ``numeric``, ``alpha_suffix``, ``entry``, and
/// ``formatted``.
///
/// Accepts the same arguments as :func:`cutter_sanborn`.
#[pyfunction]
#[pyo3(signature = (forename=None, surname=None, title=None, prefix_len=None, suffix_len=None, *, lang=None, strip_article=false))]
#[expect(
    clippy::too_many_arguments,
    reason = "one parameter per Python argument, plus the interpreter token"
)]
fn cutter_sanborn_detailed<'py>(
    py: Python<'py>,
    forename: Option<&str>,
    surname: Option<&str>,
    title: Option<&str>,
    prefix_len: Option<usize>,
    suffix_len: Option<usize>,
    lang: Option<&str>,
    strip_article: bool,
) -> PyResult<Bound<'py, PyDict>> {
    let language = article_language(lang, strip_article)?;
    let cutter = sanborn(forename, surname, title, prefix_len, suffix_len, language)?;
    let value = detailed_json(&cutter).map_err(|e| PyValueError::new_err(e.to_string()))?;
    detailed_dict(py, &value)
}

/// Return the Library of Congress cutter number as a string
/// (e.g. ``"C378"``).
///
/// At least one of ``forename``, ``surname``, or ``title`` must be
/// non-blank; blank strings count as absent. ``max_len`` defaults to 3 and
/// is clamped to 4. ``lang`` and ``strip_article`` work as in
/// :func:`cutter_sanborn`. Raises ``ValueError`` on unusable input, an
/// unknown ``lang``, or ``strip_article`` without ``lang``.
#[pyfunction]
#[pyo3(signature = (forename=None, surname=None, title=None, max_len=None, *, lang=None, strip_article=false))]
fn cutter_loc(
    forename: Option<&str>,
    surname: Option<&str>,
    title: Option<&str>,
    max_len: Option<usize>,
    lang: Option<&str>,
    strip_article: bool,
) -> PyResult<String> {
    let language = article_language(lang, strip_article)?;
    Ok(loc(forename, surname, title, max_len, language)?.to_string())
}

/// Return the parts of the Library of Congress number as a dict with the
/// keys ``alpha_prefix``, ``numeric``, and ``formatted``.
///
/// Accepts the same arguments as :func:`cutter_loc`.
#[pyfunction]
#[pyo3(signature = (forename=None, surname=None, title=None, max_len=None, *, lang=None, strip_article=false))]
fn cutter_loc_detailed<'py>(
    py: Python<'py>,
    forename: Option<&str>,
    surname: Option<&str>,
    title: Option<&str>,
    max_len: Option<usize>,
    lang: Option<&str>,
    strip_article: bool,
) -> PyResult<Bound<'py, PyDict>> {
    let language = article_language(lang, strip_article)?;
    let cutter = loc(forename, surname, title, max_len, language)?;
    let value = detailed_json(&cutter).map_err(|e| PyValueError::new_err(e.to_string()))?;
    detailed_dict(py, &value)
}

/// Run the ``csan`` command-line interface on ``args`` (excluding the
/// program name) and return the process exit code.
///
/// This is the engine behind the ``csan`` console script; regular Python
/// callers should prefer the ``cutter_*`` functions.
#[pyfunction]
fn run_cli(args: Vec<String>) -> u8 {
    csan::run_from_args(iter::once(String::from("csan")).chain(args))
}

/// Registers the extension module.
#[pymodule]
fn _csan(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(cutter_sanborn, m)?)?;
    m.add_function(wrap_pyfunction!(cutter_sanborn_detailed, m)?)?;
    m.add_function(wrap_pyfunction!(cutter_loc, m)?)?;
    m.add_function(wrap_pyfunction!(cutter_loc_detailed, m)?)?;
    m.add_function(wrap_pyfunction!(run_cli, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
