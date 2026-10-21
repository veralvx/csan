//! Text normalization for cataloging and Cutter call number generation.
//!
//! Designed to sanitize author names and titles by stripping whitespace,
//! folding diacritics, restricting to ASCII casing, and expanding cataloging
//! prefixes. Uses [`Cow`] so that already-clean input costs no allocation.

use derive_more::{Display, Error};
use std::borrow::Cow;

/// Prefixes that are traditionally normalized to `Mac` for Cutter indexing.
///
/// The Cutter-Sanborn table files every `Mc`/`M'` surname under its expanded
/// `Mac` spelling, so lookups must expand these before consulting the table.
const MAC_PREFIXES: [&str; 7] = ["Mc", "MC", "mc", "M'", "M’", "m'", "m’"];

/// Errors produced while normalizing input or generating a Cutter number.
#[derive(Debug, Display, Error)]
#[non_exhaustive]
pub enum NormalizationError {
    /// Returned when a string contains foreign characters that are not covered
    /// by the transliteration map, resulting in non-ASCII alphabetic output.
    #[display("{_0:?} is not alphabetic ASCII after trying to convert")]
    NonAsciiAlpha(#[error(ignore)] String),

    /// Returned if the input string is empty, if stripping non-alphabetic
    /// characters reduces the string to an empty state, or if an
    /// [`Author`](crate::catalog::models::Author) carries no name at all.
    #[display("empty string after trimming or converting to ASCII")]
    EmptyString,

    /// Returned when a table-driven generator is bound to an empty table.
    ///
    /// The table shipped with this crate is verified non-empty at compile
    /// time, so this variant is only reachable through a third-party
    /// [`UsesTable`](crate::catalog::cutter::generator::UsesTable)
    /// implementation that binds an empty table.
    #[display("the bound cutter table contains no entries")]
    EmptyTable,
}

use NormalizationError::{EmptyString, NonAsciiAlpha};

/// Folds a single accented alphabetic character to its base ASCII letter.
///
/// Returns the input unchanged when no mapping applies; the caller decides
/// whether an unmapped non-ASCII character is an error.
fn fold_diacritic(c: char) -> char {
    match c {
        'á' | 'à' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ą' | 'ă' => 'a',
        'Á' | 'À' | 'Â' | 'Ã' | 'Ä' | 'Å' | 'Ā' | 'Ą' | 'Ă' => 'A',

        'ç' | 'ć' | 'č' | 'ĉ' | 'ċ' => 'c',
        'Ç' | 'Ć' | 'Č' | 'Ĉ' | 'Ċ' => 'C',

        'ð' | 'ď' | 'đ' => 'd',
        'Ð' | 'Ď' | 'Đ' => 'D',

        'é' | 'è' | 'ê' | 'ë' | 'ē' | 'ę' | 'ě' | 'ĕ' | 'ė' => 'e',
        'É' | 'È' | 'Ê' | 'Ë' | 'Ē' | 'Ę' | 'Ě' | 'Ĕ' | 'Ė' => 'E',

        'ğ' | 'ġ' | 'ģ' => 'g',
        'Ğ' | 'Ġ' | 'Ģ' => 'G',

        'í' | 'ì' | 'î' | 'ï' | 'ī' | 'į' | 'ı' => 'i',
        'Í' | 'Ì' | 'Î' | 'Ï' | 'Ī' | 'Į' | 'İ' => 'I',

        'ł' | 'ľ' | 'ĺ' | 'ļ' => 'l',
        'Ł' | 'Ľ' | 'Ĺ' | 'Ļ' => 'L',

        'ñ' | 'ń' | 'ň' | 'ņ' => 'n',
        'Ñ' | 'Ń' | 'Ň' | 'Ņ' => 'N',

        'ó' | 'ò' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' | 'ő' => 'o',
        'Ó' | 'Ò' | 'Ô' | 'Õ' | 'Ö' | 'Ø' | 'Ō' | 'Ő' => 'O',

        'ř' | 'ŕ' | 'ŗ' => 'r',
        'Ř' | 'Ŕ' | 'Ŗ' => 'R',

        'ś' | 'š' | 'ş' | 'ș' => 's',
        'Ś' | 'Š' | 'Ş' | 'Ș' => 'S',

        'ť' | 'ţ' | 'ț' => 't',
        'Ť' | 'Ţ' | 'Ț' => 'T',

        'ú' | 'ù' | 'û' | 'ü' | 'ū' | 'ů' | 'ű' | 'ų' => 'u',
        'Ú' | 'Ù' | 'Û' | 'Ü' | 'Ū' | 'Ů' | 'Ű' | 'Ų' => 'U',

        'ý' | 'ÿ' => 'y',
        'Ý' | 'Ÿ' => 'Y',

        'ź' | 'ż' | 'ž' => 'z',
        'Ź' | 'Ż' | 'Ž' => 'Z',

        _ => c,
    }
}

/// Appends the ASCII expansion of `c` to `out`, if `c` is alphabetic.
///
/// Handles the ligatures and digraphs whose conventional romanization spans
/// more than one letter (`ß` → `ss`, `æ` → `ae`, `œ` → `oe`, `þ` → `th`),
/// then falls back to [`fold_diacritic`] for one-to-one mappings.
/// Non-alphabetic characters are dropped.
fn push_folded(out: &mut String, c: char) {
    match c {
        'ß' => out.push_str("ss"),
        'ẞ' => out.push_str("SS"),
        'æ' => out.push_str("ae"),
        'Æ' => out.push_str("AE"),
        'œ' => out.push_str("oe"),
        'Œ' => out.push_str("OE"),
        'þ' => out.push_str("th"),
        'Þ' => out.push_str("TH"),
        _ if c.is_alphabetic() => out.push(fold_diacritic(c)),
        _ => {}
    }
}

/// Makes a basic conversion to ASCII for the most common cases of letters with
/// diacritics or ligatures, removes non-alphabetic characters, and trims
/// whitespace.
///
/// # Errors
/// * Returns [`NormalizationError::EmptyString`] if the input is empty or if
///   removing non-alphabetic characters leaves an empty string.
/// * Returns [`NormalizationError::NonAsciiAlpha`] if the string contains
///   alphabetic characters that cannot be transliterated to a base ASCII
///   equivalent (e.g. Kanji or Cyrillic).
///
/// # Examples
/// ```
/// use std::borrow::Cow;
/// # use csan::catalog::normalization::to_ascii_alphabetic;
///
/// let clean = to_ascii_alphabetic(Cow::Borrowed("Borges")).unwrap();
/// assert!(matches!(clean, Cow::Borrowed(_))); // no allocation
///
/// let foreign = to_ascii_alphabetic(Cow::Borrowed("  Čapek  ")).unwrap();
/// assert_eq!(foreign, "Capek");
/// assert!(matches!(foreign, Cow::Owned(_)));
///
/// let ligature = to_ascii_alphabetic(Cow::Borrowed("Strauß")).unwrap();
/// assert_eq!(ligature, "Strauss");
///
/// let punctuation = to_ascii_alphabetic(Cow::Borrowed("O'Connor")).unwrap();
/// assert_eq!(punctuation, "OConnor");
/// ```
pub fn to_ascii_alphabetic(input: Cow<'_, str>) -> Result<Cow<'_, str>, NormalizationError> {
    let trimmed_input = input.trim();

    if trimmed_input.is_empty() {
        return Err(EmptyString);
    }

    for (i, c) in trimmed_input.char_indices() {
        // Also triggered by interior whitespace, which is stripped.
        if c.is_ascii_alphabetic() {
            continue;
        }

        // First divergence from "already clean": build the owned output,
        // starting from the ASCII-alphabetic prefix scanned so far. `i`
        // comes from `char_indices`, so it is always a valid boundary.
        let mut s = String::with_capacity(trimmed_input.len());
        s.push_str(trimmed_input.get(..i).unwrap_or_default());

        push_folded(&mut s, c);
        for tail in trimmed_input.get(i..).unwrap_or_default().chars().skip(1) {
            push_folded(&mut s, tail);
        }

        if s.is_empty() {
            return Err(EmptyString);
        }
        if !s.is_ascii() {
            return Err(NonAsciiAlpha(s));
        }

        return Ok(Cow::Owned(s));
    }

    if trimmed_input.len() == input.len() {
        Ok(input)
    } else {
        Ok(Cow::Owned(trimmed_input.to_owned()))
    }
}

/// Capitalizes a strictly ASCII string.
///
/// This function uppercases the first character of the string and strictly
/// lowercases all subsequent characters.
///
/// The intended input is ASCII-only, as produced by [`to_ascii_alphabetic`].
/// A string whose first character is multi-byte UTF-8 (e.g. `'É'`) is
/// returned unchanged rather than partially recased.
///
/// # Examples
/// ```
/// use std::borrow::Cow;
/// # use csan::catalog::normalization::capitalize_ascii;
///
/// let correctly_cased = capitalize_ascii(Cow::Borrowed("Doe"));
/// assert_eq!(correctly_cased, "Doe");
/// assert!(matches!(correctly_cased, Cow::Borrowed(_))); // no allocation
///
/// let badly_cased = capitalize_ascii(Cow::Borrowed("mAcDOnaLd"));
/// assert_eq!(badly_cased, "Macdonald");
/// assert!(matches!(badly_cased, Cow::Owned(_))); // allocated and fixed
/// ```
#[must_use]
pub fn capitalize_ascii(input: Cow<'_, str>) -> Cow<'_, str> {
    /// Recases `s` in place when its first character is a single byte.
    fn recase_in_place(s: &mut str) {
        if let Some((first, rest)) = s.split_at_mut_checked(1) {
            first.make_ascii_uppercase();
            rest.make_ascii_lowercase();
        }
    }

    match input {
        Cow::Owned(mut s) => {
            recase_in_place(&mut s);
            Cow::Owned(s)
        }

        Cow::Borrowed(s) => {
            let needs_alloc = match s.as_bytes().split_first() {
                Some((first_byte, rest)) => {
                    first_byte.is_ascii_lowercase() || rest.iter().any(|&b| b.is_ascii_uppercase())
                }
                None => false,
            };
            if !needs_alloc {
                return input;
            }

            let mut new_s = s.to_owned();
            recase_in_place(&mut new_s);
            Cow::Owned(new_s)
        }
    }
}

/// Standardizes Scottish and Irish patronymic prefixes for catalog sorting.
///
/// Expands recognized variations of `Mc` or `M'` to the standard `Mac`.
/// This supports traditional library filing rules, which keep phonetic
/// equivalents grouped together on the shelf, and matches the Cutter-Sanborn
/// table, which files all such names under `Mac`. Leading whitespace is
/// ignored when matching, so raw (untrimmed) input can be passed directly.
///
/// Note: traditional expansion of `St.` to `Saint` is handled at the
/// composition layer (inside
/// [`Author::composed_normalized`](crate::catalog::models::Author::composed_normalized))
/// to better manage formatting variations.
///
/// # Examples
/// ```
/// use std::borrow::Cow;
/// # use csan::catalog::normalization::normalize_mac;
/// let expanded = normalize_mac(Cow::Borrowed("M'Clintock"));
/// assert_eq!(expanded, "MacClintock");
///
/// let untrimmed = normalize_mac(Cow::Borrowed("  mcdonald"));
/// assert_eq!(untrimmed, "MacDonald");
///
/// let untouched = normalize_mac(Cow::Borrowed("Marshall"));
/// assert_eq!(untouched, "Marshall");
/// assert!(matches!(untouched, Cow::Borrowed(_)));
/// ```
#[must_use]
pub fn normalize_mac(input: Cow<'_, str>) -> Cow<'_, str> {
    let candidate = input.trim_start();
    if let Some(w) = MAC_PREFIXES.iter().find_map(|p| candidate.strip_prefix(p)) {
        return Cow::Owned(format!("Mac{}", capitalize_first_ascii(w)));
    }

    input
}

/// Uppercases the first byte of `s` when it is an ASCII letter.
///
/// Used by [`normalize_mac`] so that all-lowercase and all-uppercase inputs
/// (`mcdonald`, `MCDONALD`) both expand to a `Mac`-prefixed form whose next
/// letter survives the later [`capitalize_ascii`] pass identically.
fn capitalize_first_ascii(s: &str) -> Cow<'_, str> {
    match s.as_bytes().first() {
        Some(&b) if b.is_ascii_lowercase() => {
            let mut owned = String::with_capacity(s.len());
            owned.push(char::from(b.to_ascii_uppercase()));
            owned.push_str(s.get(1..).unwrap_or_default());
            Cow::Owned(owned)
        }
        _ => Cow::Borrowed(s),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod to_ascii_alphabetic {
        use super::*;
        #[test]
        fn basic_ascii() {
            let input = "Hello";
            let result = to_ascii_alphabetic(Cow::Borrowed(input)).unwrap();
            assert_eq!(result, "Hello");
        }

        #[test]
        fn removes_whitespace_and_symbols() {
            let input = " Hello, World! 123 ";
            let result = to_ascii_alphabetic(Cow::Borrowed(input)).unwrap();
            assert_eq!(result, "HelloWorld");
        }

        #[test]
        fn empty_error() {
            for input in ["", "   ", "12345!@#"] {
                let err = to_ascii_alphabetic(Cow::Borrowed(input)).unwrap_err();
                assert!(matches!(err, EmptyString), "input: {input:?}");
            }
        }

        #[test]
        fn unmapped_unicode_error() {
            match to_ascii_alphabetic(Cow::Borrowed("Пушкин")) {
                Err(NonAsciiAlpha(s)) => assert_eq!(s, "Пушкин"),
                other => panic!("expected NonAsciiAlpha error, got {other:?}"),
            }
        }

        #[test]
        fn mixed_valid_and_numbers() {
            let input = "User123Name";
            let result = to_ascii_alphabetic(Cow::Borrowed(input)).unwrap();
            assert_eq!(result, "UserName");
        }

        mod accents {
            use super::*;
            #[test]
            fn western_european_accents() {
                let input = "Áéîöù Çça";
                let result = to_ascii_alphabetic(Cow::Borrowed(input)).unwrap();
                assert_eq!(result, "AeiouCca");
            }

            #[test]
            fn eastern_european_accents() {
                let input = "Łąka Ńuans Život";
                let result = to_ascii_alphabetic(Cow::Borrowed(input)).unwrap();
                assert_eq!(result, "LakaNuansZivot");
            }

            #[test]
            fn nordic_accents() {
                let input = "København Århus";
                let result = to_ascii_alphabetic(Cow::Borrowed(input)).unwrap();
                assert_eq!(result, "KobenhavnArhus");
            }

            #[test]
            fn ligatures_and_digraphs() {
                let cases = [
                    ("Straße", "Strasse"),
                    ("Ærø", "AEro"),
                    ("Œuvre", "OEuvre"),
                    ("Þórður", "THordur"),
                ];
                for (input, expected) in cases {
                    let result = to_ascii_alphabetic(Cow::Borrowed(input)).unwrap();
                    assert_eq!(result, expected, "input: {input}");
                }
            }
        }
    }

    mod capitalize_ascii {
        use super::*;
        use std::borrow::Cow;

        /// Helper to reduce boilerplate when creating Cows.
        fn c(s: &str) -> Cow<'_, str> {
            Cow::from(s)
        }

        #[test]
        fn single_word_allocates() {
            let input = c("rust");
            let result = capitalize_ascii(input);

            assert_eq!(result, "Rust");
            assert!(
                matches!(result, Cow::Owned(_)),
                "input should trigger allocation"
            );
        }

        #[test]
        fn single_word_happy_path() {
            let input = c("Rust");
            let result = capitalize_ascii(input);

            assert_eq!(result, "Rust");
            assert!(
                matches!(result, Cow::Borrowed(_)),
                "input should not allocate"
            );
        }

        #[test]
        fn mixed_case_input() {
            let input = c("tHiS");
            assert_eq!(capitalize_ascii(input), "This");
        }

        #[test]
        fn multi_word_behavior() {
            assert_eq!(capitalize_ascii(c("hello world")), "Hello world");
        }

        #[test]
        fn empty() {
            let result = capitalize_ascii(c(""));
            assert_eq!(result, "");
            assert!(matches!(result, Cow::Borrowed(_)));
        }

        #[test]
        fn multibyte_head_is_returned_unchanged() {
            // Out-of-contract input must not panic (this used to split at a
            // non-UTF-8 boundary); it is handed back untouched instead.
            let odd = capitalize_ascii(c("éX"));
            assert_eq!(odd, "éX");
            let odd_owned = capitalize_ascii(Cow::Owned(String::from("Éx")));
            assert_eq!(odd_owned, "Éx");
        }

        #[test]
        fn pipeline_integration() {
            let raw = "  jOhN   ";
            let norm = to_ascii_alphabetic(Cow::Borrowed(raw)).unwrap();
            let titled = capitalize_ascii(norm);
            assert_eq!(titled, "John");
        }
    }

    mod normalize_mac {
        use super::*;

        #[test]
        fn expands_all_prefix_spellings() {
            let cases = [
                ("McDonald", "MacDonald"),
                ("mcdonald", "MacDonald"),
                ("MCDONALD", "MacDONALD"),
                ("M'Clintock", "MacClintock"),
                ("m’clintock", "MacClintock"),
            ];
            for (input, expected) in cases {
                assert_eq!(
                    normalize_mac(Cow::Borrowed(input)),
                    expected,
                    "input: {input}"
                );
            }
        }

        #[test]
        fn leading_whitespace_does_not_defeat_expansion() {
            let expanded = normalize_mac(Cow::Borrowed("  McDonald"));
            assert_eq!(expanded, "MacDonald");
        }

        #[test]
        fn full_pipeline_matches_table_casing() {
            // The table keys Mc-style surnames as e.g. "Macdon", so the
            // expansion followed by the standard pipeline must produce
            // exactly that casing.
            let piped = capitalize_ascii(
                to_ascii_alphabetic(normalize_mac(Cow::Borrowed(" MCDONALD "))).unwrap(),
            );
            assert_eq!(piped, "Macdonald");
        }

        #[test]
        fn non_mac_names_untouched() {
            let untouched = normalize_mac(Cow::Borrowed("Marshall"));
            assert_eq!(untouched, "Marshall");
            assert!(matches!(untouched, Cow::Borrowed(_)));
            // "Macbeth" already starts with Mac and must not double-expand.
            assert_eq!(normalize_mac(Cow::Borrowed("Macbeth")), "Macbeth");
        }
    }
}
