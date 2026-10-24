#![forbid(unsafe_code)]

//! Cutter call number generation for library cataloging.
//!
//! `csan-core` derives shelf-ready cutter numbers from author names (or, for
//! anonymous works, titles) using two methods:
//!
//! * **Cutter-Sanborn three-figure** — a lookup against the bundled
//!   Cutter-Sanborn table, with optional call number prefix and lowercase
//!   work mark ([`CutterSanbornMethod`]).
//! * **Library of Congress** — the `LoC` character-branching rules,
//!   producing an initial plus a digit sequence ([`CutterLocMethod`]).
//!
//! Input names are normalized first: whitespace and punctuation are
//! stripped, diacritics are folded to ASCII, `Mc`/`M'` prefixes expand to
//! `Mac`, and a surname of `St`/`Ste` expands to `Saint`/`Sainte`,
//! matching the conventions of the underlying tables.
//!
//! # Example
//! ```
//! use csan_core::{
//!     Attribution, Author, CutterConfig, CutterGenerator, CutterLocMethod,
//!     CutterSanbornMethod, Opus,
//! };
//!
//! let entry = Attribution::Both {
//!     author: Author::new("Charles", "Dickens"),
//!     opus: Opus::new("Oliver Twist"),
//! };
//!
//! let sanborn = CutterSanbornMethod::generate(&entry, CutterConfig::default())?;
//! assert_eq!(sanborn.to_string(), "D548o");
//!
//! let loc = CutterLocMethod::generate(&entry, CutterConfig::default())?;
//! assert_eq!(loc.to_string(), "D535");
//! # Ok::<(), csan_core::NormalizationError>(())
//! ```
pub mod catalog;

pub use catalog::{
    cutter::{
        CutterConfig, CutterGenerator, CutterLocMethod, CutterSanbornMethod, Detailed,
        DetailedError, detailed_json,
    },
    language::{Language, MissingLanguage, UnknownLanguage},
    models::{Attribution, Author, CutterLoc, CutterSanborn, CutterType, Opus},
    normalization::NormalizationError,
};
