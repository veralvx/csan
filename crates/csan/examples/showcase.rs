//! Prints a Markdown table of Cutter numbers for a set of well-known
//! authors, using both generation methods with default configuration.
//!
//! Run with: `cargo run --example showcase`
use csan::{
    Attribution, Author, CutterConfig, CutterGenerator, CutterLocMethod, CutterSanbornMethod,
    NormalizationError,
};
// `unused_crate_dependencies` is checked per compilation target; these
// crates are used by the library itself, just not by this example.
use {derive_more as _, serde as _, serde_json as _};

fn main() -> Result<(), NormalizationError> {
    let authors = [
        ("Jane", "Austen"),
        ("Eric", "Blair"),
        ("Agatha", "Christie"),
        ("Samuel", "Clemens"),
        ("Jorge", "De la Cruz"),
        ("Charles", "Dickens"),
        ("Emily", "Dickinson"),
        ("Fyodor", "Dostoyevsky"),
        ("Stephen", "King"),
        ("VERA", "LVX"),
        ("Herman", "Melville"),
        ("George", "Orwell"),
        ("William", "Shakespeare"),
        ("Lord", "Sith"),
        ("Ivan", "Smith"),
        ("William", "Smith"),
        ("Leo", "Tolstoy"),
        ("Mark", "Twain"),
        ("Virginia", "Woolf"),
        ("Emile", "Zola"),
    ];

    println!("| Author | Cutter-Sanborn | Library of Congress |");
    println!("| ------ | -------------- | ------------------- |");

    for (given, family) in authors {
        let entry = Attribution::AuthorOnly(Author::new(given, family));
        let sanborn = CutterSanbornMethod::generate(&entry, CutterConfig::default())?;
        let loc = CutterLocMethod::generate(&entry, CutterConfig::default())?;
        println!("| {given} {family} | `{sanborn}` | `{loc}` |");
    }

    Ok(())
}
