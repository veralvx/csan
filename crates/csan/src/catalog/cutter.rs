//! Cutter number generation: methods, configuration, and lookup tables.
pub mod detail;
pub mod generator;
pub mod tables;

pub use detail::{Detailed, DetailedError, detailed_json};
pub use generator::{
    CutterConfig, CutterGenerator, CutterLocMethod, CutterSanbornMethod, UsesTable,
};
