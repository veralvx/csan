//! Contains the logic for generating Cutter call numbers.
//!
//! The generation logic is split across three submodules:
//! * `common` — configuration ([`CutterConfig`]) and the shared
//!   [`CutterGenerator`] entry-point trait, used by both methods.
//! * `sanborn` — the Cutter-Sanborn three-figure method and its
//!   table-lookup machinery ([`CutterSanbornMethod`], [`UsesTable`]).
//! * `loc` — the Library of Congress method ([`CutterLocMethod`]) and its
//!   digit rules.
//!
//! The public items are re-exported here so callers use
//! `catalog::cutter::generator::{...}` regardless of the internal split.

mod common;
mod loc;
mod sanborn;

pub use common::{CutterConfig, CutterGenerator};
pub use loc::CutterLocMethod;
pub use sanborn::{CutterSanbornMethod, UsesTable};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_from_overrides_keeps_defaults() {
        assert_eq!(
            CutterConfig::from_overrides(None, None, None),
            CutterConfig::default()
        );
        let custom = CutterConfig::from_overrides(Some(2), None, Some(4));
        assert_eq!(
            custom,
            CutterConfig {
                prefix_len: 2,
                suffix_len: CutterConfig::default().suffix_len,
                max_len: 4,
            }
        );
    }
}
