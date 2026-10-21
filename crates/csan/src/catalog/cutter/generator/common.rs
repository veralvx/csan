//! Configuration and the shared generation entry-point trait.

use crate::catalog::models::{cutter::CutterType, entry::Attribution};
use crate::catalog::normalization::NormalizationError;

/// Tuning knobs shared by the Cutter generation methods.
///
/// Construct it with a struct literal over [`CutterConfig::default`], e.g.
/// `CutterConfig { max_len: 4, ..Default::default() }`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CutterConfig {
    /// Cutter-Sanborn: number of letters extracted from the author (or
    /// title) for the call number prefix.
    pub prefix_len: usize,
    /// Cutter-Sanborn: number of letters extracted from the title for the
    /// lowercase work mark.
    pub suffix_len: usize,
    /// `LoC`: requested number of digits. Values above
    /// [`CutterLocMethod::MAX_NUMERIC_LEN`](super::loc::CutterLocMethod::MAX_NUMERIC_LEN)
    /// are clamped down to it, and the
    /// numeral rule always produces at least the two digits of its `A15`
    /// span. A value of `0` is rejected with an error.
    pub max_len: usize,
}

impl CutterConfig {
    /// Builds a config by applying per-field overrides on top of
    /// [`CutterConfig::default`].
    ///
    /// This is the shared front door for optional user-facing tuning
    /// inputs (plugin argument maps, FFI bindings): `None` keeps the
    /// default for that field.
    #[must_use]
    pub fn from_overrides(
        prefix_len: Option<usize>,
        suffix_len: Option<usize>,
        max_len: Option<usize>,
    ) -> Self {
        let defaults = Self::default();
        Self {
            prefix_len: prefix_len.unwrap_or(defaults.prefix_len),
            suffix_len: suffix_len.unwrap_or(defaults.suffix_len),
            max_len: max_len.unwrap_or(defaults.max_len),
        }
    }
}

impl Default for CutterConfig {
    /// One prefix letter, one work mark letter, three `LoC` digits.
    fn default() -> Self {
        Self {
            prefix_len: 1, // CutterSanborn
            suffix_len: 1, // CutterSanborn
            max_len: 3,    // CutterLoc
        }
    }
}

/// Trait that unifies the Cutter generation methods behind one entry point.
pub trait CutterGenerator {
    /// The Cutter number model this method produces.
    type CutterType: CutterType;

    /// Generates a Cutter number for `entry` under `config`.
    ///
    /// # Errors
    /// Returns a [`NormalizationError`] when the entry's names cannot be
    /// normalized into table/rule input; see each implementation for the
    /// exact conditions.
    fn generate(
        entry: &Attribution,
        config: CutterConfig,
    ) -> Result<Self::CutterType, NormalizationError>;
}
