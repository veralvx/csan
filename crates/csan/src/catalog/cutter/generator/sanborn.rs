//! The Cutter-Sanborn three-figure generation method and its table-lookup
//! machinery.

use std::borrow::Cow;

use crate::catalog::cutter::tables::{CUTTER_SANBORN_TABLE, CutterNumType, TableType};
use crate::catalog::models::{author::Author, cutter::CutterSanborn, entry::Attribution};
use crate::catalog::normalization::{NormalizationError, capitalize_ascii, to_ascii_alphabetic};

use super::common::{CutterConfig, CutterGenerator};

/// A marker struct for the Cutter-Sanborn three-figure generation method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CutterSanbornMethod;

/// Defines a table binding for table-driven Cutter methods.
///
/// # Table invariants
/// [`UsesTable::TABLE`] must be sorted in ascending byte order by key, with
/// unique keys, and should be non-empty; the provided lookup relies on the
/// ordering for its binary search and reports
/// [`NormalizationError::EmptyTable`] for an empty table. The table bundled
/// with this crate ([`CUTTER_SANBORN_TABLE`]) is verified against these
/// invariants by unit tests and a compile-time non-emptiness check.
pub trait UsesTable {
    /// The sorted `(key, number)` lookup table bound to this method.
    const TABLE: TableType;

    /// Searches the static lookup table and returns the matched components.
    ///
    /// Performs an `O(log n)` binary search (`partition_point`) to find the
    /// closest matching entry within the table bound to `Self`. Multi-letter
    /// names take the last entry that is lexicographically `<=` the composed
    /// name (the *floor*), while single-letter names take the first entry
    /// above them, per Cutter-Sanborn convention.
    ///
    /// # Returns
    /// A tuple containing:
    /// * `&'static str`: The matched key from the table.
    /// * [`CutterNumType`]: The resolved numeric Cutter value.
    /// * `String`: The call number prefix — the first `prefix_len` alphabetic
    ///   characters of the normalized, composed name (shorter when the name
    ///   itself is shorter).
    ///
    /// # Errors
    /// * [`NormalizationError::EmptyString`] if the entry's name(s) normalize
    ///   to empty, or if an [`Author`] carries neither name.
    /// * [`NormalizationError::NonAsciiAlpha`] if a name contains
    ///   untransliterable characters.
    /// * [`NormalizationError::EmptyTable`] if `Self::TABLE` has no entries.
    fn retrieve_tuple(
        entry: &Attribution,
        prefix_len: usize,
    ) -> Result<(&'static str, CutterNumType, String), NormalizationError> {
        let working_name: Cow<'_, str> = match entry {
            Attribution::AuthorOnly(author) | Attribution::Both { author, .. } => {
                let family_opt = author.family_name_normalized()?;
                let given_opt = author.given_name_normalized()?;

                match (family_opt, given_opt) {
                    (Some(family), Some(given)) => {
                        // The table carries a dedicated "Smith, Wm" entry for
                        // the most common English name; route the full form
                        // there when it exists.
                        if family.as_ref() == "Smith"
                            && given.as_ref() == "William"
                            && let Ok(idx) =
                                Self::TABLE.binary_search_by_key(&"Smith, Wm", |&(key, _)| key)
                            && let Some(&(key, value)) = Self::TABLE.get(idx)
                        {
                            let prefix = alphabetic_prefix("Smith, William", prefix_len);
                            return Ok((key, value, prefix));
                        }

                        Cow::Owned(Author::compose_parts(&family, &given))
                    }
                    (Some(family), None) => family,
                    (None, Some(given)) => given,
                    (None, None) => return Err(NormalizationError::EmptyString),
                }
            }

            Attribution::OpusOnly(opus) => {
                capitalize_ascii(to_ascii_alphabetic(Cow::Borrowed(opus.title()))?)
            }
        };

        let resolved_prefix = alphabetic_prefix(&working_name, prefix_len);

        let idx = Self::TABLE.partition_point(|&(key, _)| key <= working_name.as_ref());

        let target_idx = if working_name.len() == 1 {
            idx
        } else {
            idx.saturating_sub(1)
        };

        // `partition_point <= len`, and the only way `target_idx == len` is
        // the single-letter ceiling rule on a name that sorts after every
        // key; fall back to the last entry, which equals the floor there.
        let &(key, value) = Self::TABLE
            .get(target_idx)
            .or_else(|| Self::TABLE.last())
            .ok_or(NormalizationError::EmptyTable)?;

        Ok((key, value, resolved_prefix))
    }

    /// Wrapper around [`UsesTable::retrieve_tuple`] that discards strings
    /// and returns only the resolved numeric integer.
    ///
    /// # Errors
    /// Propagates the errors of [`UsesTable::retrieve_tuple`].
    #[inline]
    fn retrieve_numeric(entry: &Attribution) -> Result<CutterNumType, NormalizationError> {
        let (_, n, _) = Self::retrieve_tuple(entry, 0)?;
        Ok(n)
    }
}

impl UsesTable for CutterSanbornMethod {
    const TABLE: TableType = {
        assert!(
            !CUTTER_SANBORN_TABLE.is_empty(),
            "the Cutter-Sanborn table must not be empty",
        );
        CUTTER_SANBORN_TABLE
    };
}

/// Collects the first `n` alphabetic characters of `name`.
///
/// Composed names carry `", "` separators; those are skipped so the prefix
/// is always pure letters (e.g. `"SmithW"` for `"Smith, William"` at
/// `n == 6`). Returns fewer characters when the name runs out.
fn alphabetic_prefix(name: &str, n: usize) -> String {
    name.chars()
        .filter(char::is_ascii_alphabetic)
        .take(n)
        .collect()
}

impl<T> CutterGenerator for T
where
    T: UsesTable,
{
    type CutterType = CutterSanborn;

    /// Constructs a [`CutterSanborn`] struct from the static table.
    ///
    /// Uses `config.prefix_len` letters from the composed name for the call
    /// number prefix and, when the entry is [`Attribution::Both`], up to
    /// `config.suffix_len` letters of the title for the lowercase work mark.
    /// The work mark is best-effort: a title that cannot be normalized, or a
    /// `suffix_len` of `0`, simply yields no work mark.
    ///
    /// # Errors
    /// * [`NormalizationError::EmptyString`] if the entry's name normalizes
    ///   to empty, or if an [`Author`] carries neither name.
    /// * [`NormalizationError::NonAsciiAlpha`] if the name contains
    ///   untransliterable characters.
    /// * [`NormalizationError::EmptyTable`] if the bound table is empty.
    ///
    /// # Examples
    /// ```
    /// # use csan::catalog::models::{author::Author, opus::Opus, entry::Attribution};
    /// # use csan::catalog::cutter::generator::{CutterGenerator, CutterSanbornMethod, CutterConfig};
    ///
    /// let author = Author::family_only("Borges");
    /// let opus = Opus::new("Ficciones");
    ///
    /// // Generates a Cutter taking 1 letter for the prefix and 1 for the suffix work mark
    /// let result =  CutterSanbornMethod::generate( &Attribution::Both {author, opus}, CutterConfig{prefix_len: 1, suffix_len: 1, ..Default::default()}).unwrap();
    ///
    /// assert_eq!(result.alpha_prefix(), "B");
    /// assert_eq!(result.numeric(), 732);
    /// assert_eq!(result.alpha_suffix(), Some("f"));
    /// assert_eq!(result.entry(), "Borg");
    /// ```
    fn generate(
        entry: &Attribution,
        config: CutterConfig,
    ) -> Result<CutterSanborn, NormalizationError> {
        let (prefix_len, suffix_len) = (config.prefix_len, config.suffix_len);
        let (matched_entry, numeric, prefix) = Self::retrieve_tuple(entry, prefix_len)?;

        let opus_for_suffix = match entry {
            Attribution::Both { opus, .. } => Some(opus),
            Attribution::AuthorOnly(_) | Attribution::OpusOnly(_) => None,
        };

        let title_suffix = opus_for_suffix
            .and_then(|o| o.as_suffix(suffix_len).ok())
            .filter(|s| !s.is_empty())
            .map(|s| {
                let mut owned = s.into_owned();
                owned.make_ascii_lowercase();
                owned
            });

        Ok(CutterSanborn::new(
            prefix,
            numeric,
            title_suffix,
            matched_entry,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::catalog::models::Opus;

    use super::*;

    #[test]
    fn cutter_sanborn_general() {
        let cases = [
            // Misc cases
            ("Charles", "Dickens", 548),
            ("Jane", "Austen", 933),
            ("Mark", "Twain", 969),
            ("Samuel", "Clemens", 625),
            ("George", "Orwell", 79),
            ("Eric", "Blair", 635),
            ("Virginia", "Woolf", 913),
            ("Leo", "Tolstoy", 654),
            ("Fyodor", "Dostoyevsky", 724),
            ("Herman", "Melville", 531),
            ("Emily", "Dickinson", 553),
            ("William", "Shakespeare", 527),
            ("Agatha", "Christie", 555),
            ("Stephen", "King", 54),
            ("Jorge", "De la Cruz", 332),
            ("Mary", "Abbot", 128),
            ("M", "Abbott", 133),
            ("M.", "Abbott", 133),
            ("M.", "Abbottz", 134),
            ("M.", "Abbota", 129),
            ("M.", "Abbotta", 134),
            ("Mary", "Abbott", 133),
            ("Ivan", "Smith", 649),
            ("Ivan.", "Smith", 649),
            ("I", "Smith", 649),
            ("I.", "Smith", 649),
            ("John", "Smith", 652),
            ("Johnny", "Smith", 652),
            ("Jos.", "Smith", 653),
            ("Joseph", "Smith", 653),
            ("Roberto", "Smith", 658),
            ("Robert", "Smith", 658),
            ("Sol", "Smith", 661),
            ("Sophomore", "Smith", 661),
            ("Solstice", "Smith", 661),
            ("Will", "Smith", 663),
            ("William", "Smith", 664),
            ("Wm", "Smith", 664),
            ("Wmayo", "Smith", 664),
            ("VERA", "LVX", 979),
            ("Lord", "Sith", 623),
            ("Emile", "Zola", 86),
            ("Jane", "O'Neil", 58),
            ("Monica", "Santiago", 235),
            // Saint cases
            ("Alfred", "San", 194),
            ("Francisco", "San", 195),
            ("Alfred", "Saint", 133),
            ("Paul", "St.", 149),
            ("Paul", "St", 149),
            ("Klaus", "Saint", 144),
            ("Claus", "San", 194),
            ("San", "San", 197),
            ("Zarathustra", "San", 197),
            ("Santa", "Claus", 616),
        ];

        let none_cases = [
            ("San", 194),
            ("Sanculotte", 212),
            ("Aarabc", 113),
            ("Aa", 111),
            ("Aabbcc", 111),
            ("Ba", 111),
            ("Baab", 111),
            ("Cca", 386),
            ("Xz", 9),
            ("Za", 11),
            ("Zaaz", 11),
            ("Zy", 99),
            ("Zyz", 99),
            ("Zz", 99),
            ("A", 111),
            ("B", 111),
            ("X", 1),
            ("Y", 11),
            ("Z", 11),
        ];

        for (first, last, expected) in cases {
            let author = Author::new(first, last);
            let num =
                CutterSanbornMethod::retrieve_numeric(&Attribution::AuthorOnly(author)).unwrap();

            assert_eq!(num, expected, "Failed for Author: {first:?} {last}");
        }

        for (name, expected) in none_cases {
            let author = Author::family_only(name);
            let num =
                CutterSanbornMethod::retrieve_numeric(&Attribution::AuthorOnly(author)).unwrap();
            assert_eq!(num, expected, "Failed for Author: {name:?}");
        }
    }

    #[test]
    fn all_entries_cutter_sanborn() {
        for &(k, v) in CUTTER_SANBORN_TABLE {
            let attribution = match k.split_once(", ") {
                Some((family, given)) => Attribution::AuthorOnly(Author::new(given, family)),
                None => match k.split_once(' ') {
                    Some((p1, p2)) if matches!(p1, "Saint" | "Sainte" | "San") => {
                        Attribution::AuthorOnly(Author::new(p2, p1))
                    }
                    _ => Attribution::AuthorOnly(Author::family_only(k)),
                },
            };

            assert_eq!(
                CutterSanbornMethod::retrieve_numeric(&attribution).unwrap(),
                v,
                "Failed for entry: {k}"
            );
        }
    }

    #[test]
    fn table_is_sorted_and_unique() {
        // Guards against a bad table regeneration: the real table has
        // over 12k entries, and strict ascending order implies unique
        // keys.
        assert!(CUTTER_SANBORN_TABLE.len() > 12_000, "table truncated?");
        assert!(
            CUTTER_SANBORN_TABLE.is_sorted_by(|a, b| a.0 < b.0),
            "table keys must be strictly ascending"
        );
    }

    #[test]
    fn every_letter_has_an_entry_above_it() {
        // The single-letter ceiling rule indexes at `partition_point`
        // without subtracting; this invariant keeps that lookup in
        // bounds for every letter A-Z.
        for letter in b'A'..=b'Z' {
            let s = char::from(letter).to_string();
            let idx = CUTTER_SANBORN_TABLE.partition_point(|&(k, _)| k <= s.as_str());
            assert!(idx < CUTTER_SANBORN_TABLE.len(), "letter {s}");
        }
    }

    #[test]
    fn prefix_is_alphabetic_and_bounded() {
        // Special-cased "Smith, Wm" table entry.
        let smith = Attribution::AuthorOnly(Author::new("William", "Smith"));
        let (entry, numeric, p3) = CutterSanbornMethod::retrieve_tuple(&smith, 3).unwrap();
        assert_eq!((entry, numeric, p3.as_str()), ("Smith, Wm", 664, "Smi"));

        let (_, _, p6) = CutterSanbornMethod::retrieve_tuple(&smith, 6).unwrap();
        assert_eq!(p6, "SmithW", "the ', ' separator is skipped");

        let (_, _, p_long) = CutterSanbornMethod::retrieve_tuple(&smith, 64).unwrap();
        assert_eq!(p_long, "SmithWilliam", "over-long prefixes must not panic");

        // Regular composed name.
        let robert = Attribution::AuthorOnly(Author::new("Robert", "Smith"));
        let (_, _, p8) = CutterSanbornMethod::retrieve_tuple(&robert, 8).unwrap();
        assert_eq!(p8, "SmithRob");

        // Title-derived entries used to panic on over-long prefixes.
        let opus = Attribution::OpusOnly(Opus::new("Öl!"));
        let (_, _, p5) = CutterSanbornMethod::retrieve_tuple(&opus, 5).unwrap();
        assert_eq!(p5, "Ol");
    }

    #[test]
    fn author_with_no_names_errors() {
        let author: Author = serde_json::from_str("{}").unwrap();
        let attr = Attribution::AuthorOnly(author);
        assert!(matches!(
            CutterSanbornMethod::retrieve_numeric(&attr),
            Err(NormalizationError::EmptyString)
        ));
    }

    #[test]
    fn given_only_author_matches_family_only_lookup() {
        let via_given = CutterSanbornMethod::retrieve_numeric(&Attribution::AuthorOnly(
            Author::given_only("Prince"),
        ))
        .unwrap();
        let via_family = CutterSanbornMethod::retrieve_numeric(&Attribution::AuthorOnly(
            Author::family_only("Prince"),
        ))
        .unwrap();
        assert_eq!(via_given, via_family);
    }

    #[test]
    fn mac_prefix_names_hit_expanded_entries() {
        for name in ["MacClintock", "M'Clintock", "McClintock", " mcclintock "] {
            let num = CutterSanbornMethod::retrieve_numeric(&Attribution::AuthorOnly(
                Author::family_only(name),
            ))
            .unwrap();
            assert_eq!(num, 127, "name {name:?}");
        }
    }

    #[test]
    fn work_mark_is_best_effort() {
        let both = |title: &str| Attribution::Both {
            author: Author::family_only("Dickens"),
            opus: Opus::new(title),
        };

        let ok =
            CutterSanbornMethod::generate(&both("Oliver Twist"), CutterConfig::default()).unwrap();
        assert_eq!(ok.to_string(), "D548o");

        // Unnormalizable title: the number is still generated, without a
        // work mark.
        let no_mark = CutterSanbornMethod::generate(&both("!!!"), CutterConfig::default()).unwrap();
        assert_eq!(no_mark.alpha_suffix(), None);
        assert_eq!(no_mark.numeric(), 548);

        // suffix_len == 0 yields no work mark rather than an empty one.
        let zero = CutterSanbornMethod::generate(
            &both("Oliver"),
            CutterConfig {
                suffix_len: 0,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(zero.alpha_suffix(), None);
        assert_eq!(zero.to_string(), "D548");
    }

    #[test]
    fn cutter_sanborn_struct() {
        let cases = [
            ("S", 289, Some("t"), "S289t"),
            ("Sa", 11, Some("ta"), "Sa11ta"),
            ("S", 289, None, "S289"),
        ];
        for (prefix, numeric, suffix, expected) in cases {
            let cutter_struct =
                CutterSanborn::new(prefix.into(), numeric, suffix.map(Into::into), "Mock");
            assert_eq!(format!("{cutter_struct}"), expected);
        }
    }
}
