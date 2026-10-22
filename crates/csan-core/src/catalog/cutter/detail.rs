//! The canonical *detailed* representation of a generated cutter number.
//!
//! Every frontend (CLI JSON, Python dicts, the web/JS bindings, and the
//! Typst plugin) exposes the same shape: the cutter's component fields —
//! `alpha_prefix`, `numeric`, and, for Cutter-Sanborn, `alpha_suffix` and
//! `entry` — plus the rendered call number under `formatted`.
//!
//! Rather than reimplement that shape per frontend, the shape lives here as
//! two reusable pieces:
//!
//! * [`Detailed`] — a serde adapter that serializes any cutter value (a
//!   [`serde::Serialize`] type whose [`Display`] is its rendered form) as its
//!   own fields with `formatted` appended. It is format-agnostic, so each
//!   frontend serializes it in its native format (JSON, CBOR, a JS object).
//! * [`detailed_json`] — a convenience that runs [`Detailed`] through
//!   `serde_json` and returns the resulting object, used by frontends that
//!   speak JSON.
//!
//! [`CutterLoc`](crate::catalog::models::cutter::CutterLoc) and
//! [`CutterSanborn`](crate::catalog::models::cutter::CutterSanborn) both
//! satisfy the `Serialize + Display` bound, so
//! `Detailed::new(&cutter)` is the one place the detailed shape is defined.

use std::fmt::Display;

use derive_more::{Display as DeriveDisplay, Error as DeriveError};
use serde::ser::{Error as SerError, Serialize, SerializeMap, Serializer};

/// A serde adapter that presents a cutter value as its component fields plus
/// the rendered call number under `formatted`.
///
/// The wrapped value must serialize as a map (all cutter models are structs,
/// so they do) and its [`Display`] output is used verbatim as `formatted`.
/// Serializing `Detailed` in any serde format yields the frontend-agnostic
/// detailed shape; see the [module docs](self) for the field list.
///
/// `formatted` is appended after the wrapped value's own fields. The cutter
/// models have no field of that name, so there is no conflict; if a custom
/// wrapped type already defines `formatted`, the appended entry shadows it
/// (last-wins in JSON; a duplicate key in formats that permit one, such as
/// CBOR).
///
/// # Examples
/// ```
/// use csan_core::catalog::cutter::detail::Detailed;
/// use csan_core::CutterLoc;
///
/// let cutter = CutterLoc::new('C', 378);
/// let json = serde_json::to_value(Detailed::new(&cutter)).unwrap();
///
/// assert_eq!(json["alpha_prefix"], "C");
/// assert_eq!(json["numeric"], 378);
/// assert_eq!(json["formatted"], "C378");
/// ```
#[derive(Debug)]
pub struct Detailed<'a, T> {
    value: &'a T,
}

// `Detailed` only holds a shared reference, so it is unconditionally `Copy`
// (and `Clone`) regardless of `T`. Deriving these would wrongly add a
// `T: Copy` / `T: Clone` bound, making `Detailed<'_, CutterSanborn>` (whose
// `T` is not `Copy`) needlessly non-`Copy`; the manual impls avoid that.
impl<T> Clone for Detailed<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Detailed<'_, T> {}

impl<'a, T> Detailed<'a, T> {
    /// Wraps a cutter value for detailed serialization.
    #[must_use]
    pub fn new(value: &'a T) -> Self {
        Self { value }
    }
}

impl<T> Serialize for Detailed<'_, T>
where
    T: Serialize + Display,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Collect the wrapped value's own fields first so `formatted` can be
        // appended as an additional entry. Cutter models are small structs,
        // so the intermediate `serde_json::Value` is cheap and lets this
        // adapter stay generic over the output format.
        let base = serde_json::to_value(self.value).map_err(SerError::custom)?;

        let serde_json::Value::Object(fields) = base else {
            return Err(SerError::custom(
                "Detailed can only wrap values that serialize as a map",
            ));
        };

        let mut map = serializer.serialize_map(Some(fields.len().saturating_add(1)))?;
        for (key, field_value) in &fields {
            map.serialize_entry(key, field_value)?;
        }
        map.serialize_entry("formatted", &self.value.to_string())?;
        map.end()
    }
}

/// The error returned by [`detailed_json`] when a value cannot be rendered
/// into the detailed JSON object.
///
/// In practice the cutter models always serialize as JSON objects, so this
/// is defensive: it is only produced if [`detailed_json`] is called on a type
/// that serializes as a non-object (which [`Detailed`] reports as a
/// serialization error) or fails to serialize at all.
#[derive(Debug, DeriveDisplay, DeriveError)]
#[non_exhaustive]
pub enum DetailedError {
    /// The value could not be serialized into the detailed object — either it
    /// failed to serialize at all, or it did not serialize as a map (so there
    /// were no fields to attach `formatted` to). Wraps the originating
    /// [`serde_json::Error`] as its source.
    #[display("failed to serialize cutter value: {_0}")]
    Serialize(serde_json::Error),
}

/// Serializes a cutter value into a JSON object of its parts, with the
/// rendered call number added under `formatted`.
///
/// This is the shared implementation behind the frontends that emit JSON
/// (currently the CLI's `--json` output and, via a small bridge, the Python
/// `*_detailed` dicts). It is a thin `serde_json` wrapper over [`Detailed`].
///
/// # Errors
/// Returns [`DetailedError`] if `value` cannot be serialized to JSON or does
/// not serialize as a JSON object. Unreachable for the cutter models, which
/// are structs.
///
/// # Examples
/// ```
/// use csan_core::catalog::cutter::detail::detailed_json;
/// use csan_core::CutterSanborn;
///
/// let cutter = CutterSanborn::new("D".to_owned(), 548, Some("o".to_owned()), "Dickens");
/// let object = detailed_json(&cutter).unwrap();
///
/// assert_eq!(object["alpha_prefix"], "D");
/// assert_eq!(object["numeric"], 548);
/// assert_eq!(object["alpha_suffix"], "o");
/// assert_eq!(object["entry"], "Dickens");
/// assert_eq!(object["formatted"], "D548o");
/// ```
pub fn detailed_json<T>(value: &T) -> Result<serde_json::Value, DetailedError>
where
    T: Serialize + Display,
{
    serde_json::to_value(Detailed::new(value)).map_err(DetailedError::Serialize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::models::cutter::{CutterLoc, CutterSanborn};
    use std::error::Error as _;

    #[test]
    fn detailed_json_loc_has_parts_and_formatted() {
        let cutter = CutterLoc::new('C', 378);
        let object = detailed_json(&cutter).unwrap();

        assert_eq!(object["alpha_prefix"], "C");
        assert_eq!(object["numeric"], 378);
        assert_eq!(object["formatted"], "C378");
        // Exactly the parts plus `formatted`, nothing else.
        assert_eq!(object.as_object().unwrap().len(), 3);
    }

    #[test]
    fn detailed_json_sanborn_has_all_parts_and_formatted() {
        let cutter = CutterSanborn::new("D".to_owned(), 548, Some("o".to_owned()), "Dickens");
        let object = detailed_json(&cutter).unwrap();

        assert_eq!(object["alpha_prefix"], "D");
        assert_eq!(object["numeric"], 548);
        assert_eq!(object["alpha_suffix"], "o");
        assert_eq!(object["entry"], "Dickens");
        assert_eq!(object["formatted"], "D548o");
        assert_eq!(object.as_object().unwrap().len(), 5);
    }

    #[test]
    fn sanborn_without_work_mark_keeps_null_suffix() {
        let cutter = CutterSanborn::new("B".to_owned(), 732, None, "Borg");
        let object = detailed_json(&cutter).unwrap();

        assert_eq!(object["alpha_prefix"], "B");
        assert_eq!(object["alpha_suffix"], serde_json::Value::Null);
        assert_eq!(object["entry"], "Borg");
        assert_eq!(object["formatted"], "B732");
    }

    #[test]
    fn detailed_wrapper_matches_detailed_json() {
        // `detailed_json` is defined as `Detailed` through serde_json; the
        // two must agree field for field.
        let cutter = CutterSanborn::new("D".to_owned(), 548, Some("o".to_owned()), "Dickens");
        let via_wrapper = serde_json::to_value(Detailed::new(&cutter)).unwrap();
        let via_helper = detailed_json(&cutter).unwrap();

        assert_eq!(via_wrapper, via_helper);
    }

    #[test]
    fn numeric_stays_an_integer_not_a_float() {
        // The frontends (Python `int`, CBOR integer) depend on `numeric`
        // round-tripping as an integer rather than a float.
        let object = detailed_json(&CutterLoc::new('C', 378)).unwrap();
        assert!(object["numeric"].is_u64(), "numeric must remain an integer");
        assert!(
            !object["numeric"].is_f64(),
            "numeric must not become a float"
        );
    }

    #[test]
    fn detailed_json_rejects_non_object_values() {
        // `Detailed`/`detailed_json` are generic; a value that does not
        // serialize as a map has no fields to attach `formatted` to. The
        // `Detailed` adapter reports this as a serialization failure, so
        // `detailed_json` surfaces `Serialize` rather than panicking or
        // producing a bare scalar. (The cutter models are structs, so this
        // path is only reachable through this kind of misuse.)
        let error = detailed_json(&42_u32).unwrap_err();
        assert!(matches!(error, DetailedError::Serialize(_)));
        // The error renders a useful message and carries its source.
        assert!(!error.to_string().is_empty());
        assert!(error.source().is_some());
    }

    #[test]
    fn formatted_is_the_final_key() {
        // `formatted` is appended after the wrapped value's own fields; the
        // frontends document it as the rendered call number alongside the
        // parts, so it must always be present.
        let object = detailed_json(&CutterLoc::new('C', 378)).unwrap();
        assert!(object.get("formatted").is_some());
    }

    #[test]
    fn detailed_is_copy_even_when_wrapped_value_is_not() {
        // `Detailed` only holds a shared reference, so it must be `Copy`
        // regardless of `T`. `CutterSanborn` is not `Copy`; wrapping it must
        // still yield a `Copy` adapter (deriving `Copy` would have added a
        // spurious `T: Copy` bound and broken this).
        fn assert_copy<T: Copy>(_: &T) {}
        let cutter = CutterSanborn::new("D".to_owned(), 548, Some("o".to_owned()), "Dickens");
        let detailed = Detailed::new(&cutter);
        let copied = detailed;
        assert_copy(&detailed);
        // Both bindings remain usable, confirming a copy (not a move).
        assert_eq!(
            serde_json::to_value(detailed).unwrap(),
            serde_json::to_value(copied).unwrap()
        );
    }
}
