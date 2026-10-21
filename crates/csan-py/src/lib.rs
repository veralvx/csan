// The pyo3 macro expansions contain `unsafe`, so
// `#![forbid(unsafe_code)]` (which cannot be overridden) is not usable
// here. `deny` still rejects any hand-written `unsafe` in this crate
// while allowing the macro-generated blocks through.
#![deny(unsafe_code)]

//! The `csan._csan` Python extension module.
//!
//! Exposes [`csan`]'s Cutter call number generation to Python, plus the
//! `run_cli` entry point that powers the `csan` console script (the same
//! `clap` interface as the standalone binary, provided by [`csan_cli`]).
//! The user-facing package (docstrings, `main()`, re-exports) lives in
//! `python/csan/__init__.py`.

use csan::{
    Attribution, CutterConfig, CutterGenerator, CutterLoc, CutterLocMethod, CutterSanborn,
    CutterSanbornMethod, detailed_json,
};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use serde_json::Value;
use std::fmt;
use std::iter;

/// Builds the [`Attribution`] for a set of optional inputs, with a
/// Python-friendly error.
fn attribution(
    given_name: Option<&str>,
    family_name: Option<&str>,
    title: Option<&str>,
) -> PyResult<Attribution> {
    Attribution::from_parts(given_name, family_name, title).ok_or_else(|| {
        PyValueError::new_err(
            "provide at least one non-blank input: given_name, family_name, or title",
        )
    })
}

/// Maps a generation failure to a Python `ValueError`.
fn generation_error(error: &impl fmt::Display) -> PyErr {
    PyValueError::new_err(format!("cutter generation failed: {error}"))
}

/// Generates the Cutter-Sanborn value shared by both `cutter_sanborn`
/// variants.
fn sanborn(
    given_name: Option<&str>,
    family_name: Option<&str>,
    title: Option<&str>,
    prefix_len: Option<usize>,
    suffix_len: Option<usize>,
) -> PyResult<CutterSanborn> {
    let entry = attribution(given_name, family_name, title)?;
    let config = CutterConfig::from_overrides(prefix_len, suffix_len, None);
    CutterSanbornMethod::generate(&entry, config).map_err(|e| generation_error(&e))
}

/// Generates the `LoC` value shared by both `cutter_loc` variants.
fn loc(
    given_name: Option<&str>,
    family_name: Option<&str>,
    title: Option<&str>,
    max_len: Option<usize>,
) -> PyResult<CutterLoc> {
    let entry = attribution(given_name, family_name, title)?;
    let config = CutterConfig::from_overrides(None, None, max_len);
    CutterLocMethod::generate(&entry, config).map_err(|e| generation_error(&e))
}

/// Return the Cutter-Sanborn call number as a string (e.g. ``"D548o"``).
///
/// At least one of ``given_name``, ``family_name``, or ``title`` must be
/// non-blank; blank strings count as absent. ``prefix_len`` and
/// ``suffix_len`` default to 1. Raises ``ValueError`` on unusable input.
#[pyfunction]
#[pyo3(signature = (given_name=None, family_name=None, title=None, prefix_len=None, suffix_len=None))]
fn cutter_sanborn(
    given_name: Option<&str>,
    family_name: Option<&str>,
    title: Option<&str>,
    prefix_len: Option<usize>,
    suffix_len: Option<usize>,
) -> PyResult<String> {
    Ok(sanborn(given_name, family_name, title, prefix_len, suffix_len)?.to_string())
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

#[pyfunction]
#[pyo3(signature = (given_name=None, family_name=None, title=None, prefix_len=None, suffix_len=None))]
fn cutter_sanborn_detailed<'py>(
    py: Python<'py>,
    given_name: Option<&str>,
    family_name: Option<&str>,
    title: Option<&str>,
    prefix_len: Option<usize>,
    suffix_len: Option<usize>,
) -> PyResult<Bound<'py, PyDict>> {
    let cutter = sanborn(given_name, family_name, title, prefix_len, suffix_len)?;
    let value = detailed_json(&cutter).map_err(|e| PyValueError::new_err(e.to_string()))?;
    detailed_dict(py, &value)
}

/// Return the Library of Congress cutter number as a string
/// (e.g. ``"C378"``).
///
/// At least one of ``given_name``, ``family_name``, or ``title`` must be
/// non-blank; blank strings count as absent. ``max_len`` defaults to 3 and
/// is clamped to 4. Raises ``ValueError`` on unusable input.
#[pyfunction]
#[pyo3(signature = (given_name=None, family_name=None, title=None, max_len=None))]
fn cutter_loc(
    given_name: Option<&str>,
    family_name: Option<&str>,
    title: Option<&str>,
    max_len: Option<usize>,
) -> PyResult<String> {
    Ok(loc(given_name, family_name, title, max_len)?.to_string())
}

/// Return the parts of the Library of Congress number as a dict with the
/// keys ``alpha_prefix``, ``numeric``, and ``formatted``.
///
/// Accepts the same arguments as :func:`cutter_loc`.
#[pyfunction]
#[pyo3(signature = (given_name=None, family_name=None, title=None, max_len=None))]
fn cutter_loc_detailed<'py>(
    py: Python<'py>,
    given_name: Option<&str>,
    family_name: Option<&str>,
    title: Option<&str>,
    max_len: Option<usize>,
) -> PyResult<Bound<'py, PyDict>> {
    let cutter = loc(given_name, family_name, title, max_len)?;
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
    csan_cli::run_from_args(iter::once(String::from("csan")).chain(args))
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
