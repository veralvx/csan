//! Defines the generated Cutter number models.
use derive_more::Display;
use std::fmt;

use crate::catalog::cutter::tables::CutterNumType;

/// Represents a generated Library of Congress (`LoC`) Cutter number.
///
/// `LoC` Cutters always consist of a single initial alphabetic character
/// followed by a numeric sequence derived from character-branching rules.
///
/// The `Display` implementation outputs the standard concatenated format
/// (e.g. `C378` or `A153`).
#[derive(Display)]
#[display("{alpha_prefix}{numeric}")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct CutterLoc {
    /// The initial letter of the cuttered name (e.g. `'C'`).
    alpha_prefix: char,
    /// The digit sequence derived from the `LoC` branching rules.
    numeric: CutterNumType,
}

impl CutterLoc {
    /// Creates a new `CutterLoc` instance.
    ///
    /// # Examples
    /// ```
    /// # use csan_core::catalog::models::cutter::CutterLoc;
    ///
    /// let loc_cutter = CutterLoc::new('C', 378);
    /// assert_eq!(loc_cutter.to_string(), "C378");
    /// ```
    #[must_use]
    pub fn new(alpha_prefix: char, numeric: CutterNumType) -> CutterLoc {
        CutterLoc {
            alpha_prefix,
            numeric,
        }
    }

    /// Returns the initial letter of the Cutter number.
    #[must_use]
    pub fn alpha_prefix(&self) -> char {
        self.alpha_prefix
    }

    /// Returns the numeric portion of the Cutter number.
    #[must_use]
    pub fn numeric(&self) -> CutterNumType {
        self.numeric
    }
}

/// Represents a generated Cutter-Sanborn call number.
///
/// The `Display` implementation outputs the standard concatenated format
/// with the optional lowercase work mark appended (e.g. `D548` or `D548o`).
#[derive(Display)]
#[display("{}{}{}", alpha_prefix, numeric, alpha_suffix.as_deref().unwrap_or(""))]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct CutterSanborn {
    /// The alphabetic prefix taken from the author (or title) name,
    /// e.g. `"D"` for Dickens.
    alpha_prefix: String,
    /// The number associated with the matched Cutter-Sanborn table entry.
    numeric: CutterNumType,
    /// The optional lowercase work mark derived from the title,
    /// e.g. `"o"` for *Oliver Twist*.
    alpha_suffix: Option<String>,
    /// The Cutter-Sanborn table entry that was matched (e.g. `"Dickens"`).
    entry: &'static str,
}

impl CutterSanborn {
    /// Creates a new `CutterSanborn` instance from its parts.
    #[must_use]
    pub fn new(
        alpha_prefix: String,
        numeric: CutterNumType,
        alpha_suffix: Option<String>,
        entry: &'static str,
    ) -> CutterSanborn {
        CutterSanborn {
            alpha_prefix,
            numeric,
            alpha_suffix,
            entry,
        }
    }

    /// Returns the alphabetic prefix of the call number.
    #[must_use]
    pub fn alpha_prefix(&self) -> &str {
        &self.alpha_prefix
    }

    /// Returns the lowercase work mark, if one was generated.
    #[must_use]
    pub fn alpha_suffix(&self) -> Option<&str> {
        self.alpha_suffix.as_deref()
    }

    /// Returns the numeric portion of the call number.
    #[must_use]
    pub fn numeric(&self) -> CutterNumType {
        self.numeric
    }

    /// Returns the Cutter-Sanborn table entry that was matched.
    #[must_use]
    pub fn entry(&self) -> &'static str {
        self.entry
    }
}

/// Marker trait for the value types a
/// [`CutterGenerator`](crate::catalog::cutter::CutterGenerator) can produce.
///
/// Every cutter model is displayable as its call number string.
pub trait CutterType: fmt::Display {}
impl CutterType for CutterSanborn {}
impl CutterType for CutterLoc {}
