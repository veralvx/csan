//! Data models for catalog entries and generated Cutter numbers.
pub mod author;
pub mod cutter;
pub mod entry;
pub mod opus;

pub use {
    author::Author,
    cutter::{CutterLoc, CutterSanborn, CutterType},
    entry::Attribution,
    opus::Opus,
};
