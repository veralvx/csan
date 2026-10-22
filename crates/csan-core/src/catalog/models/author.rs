//! Defines the [`Author`] struct model.
use std::borrow::Cow;

use crate::catalog::normalization::{
    NormalizationError, capitalize_ascii, normalize_mac, to_ascii_alphabetic,
};

/// Represents an author in the catalog, consisting of an optional given name
/// and an optional family name.
///
/// At least one of the two names must be present for the author to be usable
/// by a cutter generator; an author with neither yields
/// [`NormalizationError::EmptyString`] at generation time. This state is not
/// constructible through [`Author::new`], [`Author::surname_only`], or
/// [`Author::forename_only`], but can arise through deserialization.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Author {
    /// The author's given (first) name, e.g. `"William"`.
    #[serde(default)]
    forename: Option<String>,
    /// The author's family (last) name, e.g. `"Shakespeare"`.
    #[serde(default)]
    surname: Option<String>,
}

impl Author {
    /// Creates a new `Author` with both a given name and a family name.
    ///
    /// # Examples
    /// ```
    /// # use csan_core::catalog::models::author::Author;
    /// let author = Author::new("William", "Shakespeare");
    /// ```
    #[must_use]
    pub fn new(forename: impl Into<String>, surname: impl Into<String>) -> Self {
        Self {
            forename: Some(forename.into()),
            surname: Some(surname.into()),
        }
    }

    /// Creates an `Author` with only a family name.
    #[must_use]
    pub fn surname_only(surname: impl Into<String>) -> Self {
        Self {
            forename: None,
            surname: Some(surname.into()),
        }
    }

    /// Creates an `Author` with only a given name.
    #[must_use]
    pub fn forename_only(forename: impl Into<String>) -> Self {
        Self {
            forename: Some(forename.into()),
            surname: None,
        }
    }

    /// Returns the author's raw (unnormalized) given name, if present.
    #[inline]
    #[must_use]
    pub fn forename(&self) -> Option<&str> {
        self.forename.as_deref()
    }

    /// Returns the author's raw (unnormalized) family name, if present.
    #[inline]
    #[must_use]
    pub fn surname(&self) -> Option<&str> {
        self.surname.as_deref()
    }

    /// Retrieves the given name and applies normalization.
    ///
    /// Returns `Ok(None)` if the author does not have a given name.
    ///
    /// # Errors
    /// Returns a [`NormalizationError`] if a present name fails
    /// [`Author::normalize_name`].
    pub fn forename_normalized(&self) -> Result<Option<Cow<'_, str>>, NormalizationError> {
        self.forename().map(Author::normalize_name).transpose()
    }

    /// Retrieves the family name and applies normalization.
    ///
    /// Returns `Ok(None)` if the author does not have a family name.
    ///
    /// # Errors
    /// Returns a [`NormalizationError`] if a present name fails
    /// [`Author::normalize_name`].
    pub fn surname_normalized(&self) -> Result<Option<Cow<'_, str>>, NormalizationError> {
        self.surname().map(Author::normalize_name).transpose()
    }

    /// Applies the full normalization pipeline to a raw name.
    ///
    /// # Steps
    /// 1. Expands Scottish/Irish `Mc`/`M'` prefixes to `Mac`
    ///    ([`normalize_mac`]).
    /// 2. Flattens diacritics and strips non-alphabetic characters
    ///    ([`to_ascii_alphabetic`]).
    /// 3. Capitalizes the first letter and lowercases the remainder
    ///    ([`capitalize_ascii`]).
    ///
    /// # Errors
    /// Returns a [`NormalizationError`] if the string becomes empty after
    /// stripping, or if untransliterable non-ASCII characters remain.
    pub fn normalize_name(input: &str) -> Result<Cow<'_, str>, NormalizationError> {
        let norm_mac = normalize_mac(Cow::Borrowed(input));
        let normalized = to_ascii_alphabetic(norm_mac)?;
        Ok(capitalize_ascii(normalized))
    }

    /// Composes a single normalized string from a given name and a family
    /// name.
    ///
    /// # Rules
    /// * Standard names are formatted as `"Family, Given"`
    ///   (e.g. `"Smith, William"`), matching the Cutter-Sanborn table's key
    ///   format.
    /// * Hagionyms keep filing order without the comma: a family name of
    ///   `Saint`, `Sainte`, or `San` produces e.g. `"Saint Thomas"`, and the
    ///   abbreviations `St`/`Ste` (including `St.`/`Ste.`, whose period is
    ///   stripped by normalization) are expanded to their full forms.
    ///
    /// # Errors
    /// Returns a [`NormalizationError`] if either name fails normalization.
    pub fn composed_normalized<T, U>(given: T, family: U) -> Result<String, NormalizationError>
    where
        T: AsRef<str>,
        U: AsRef<str>,
    {
        let given = Author::normalize_name(given.as_ref())?;
        let family = Author::normalize_name(family.as_ref())?;
        Ok(Author::compose_parts(&family, &given))
    }

    /// Composes two *already normalized* name parts into a table lookup key.
    ///
    /// This is the composition step of [`Author::composed_normalized`],
    /// split out so that callers holding normalized parts (e.g. the cutter
    /// generators) do not pay for a second normalization pass.
    #[must_use]
    pub(crate) fn compose_parts(surname_normalized: &str, forename_normalized: &str) -> String {
        match surname_normalized {
            "Saint" | "Sainte" | "San" => format!("{surname_normalized} {forename_normalized}"),
            "St" => format!("Saint {forename_normalized}"),
            "Ste" => format!("Sainte {forename_normalized}"),
            _ => format!("{surname_normalized}, {forename_normalized}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::normalization::NormalizationError;
    use std::borrow::Cow;

    mod constructor {
        use super::*;

        #[test]
        fn new_with_full_name() {
            let author = Author::new("John", "Doe");
            assert_eq!(author.forename(), Some("John"));
            assert_eq!(author.surname(), Some("Doe"));
        }

        #[test]
        fn new_with_no_forename() {
            let author = Author::surname_only("Plato");
            assert_eq!(author.forename(), None);
            assert_eq!(author.surname(), Some("Plato"));
        }

        #[test]
        fn new_with_no_surname() {
            let author = Author::forename_only("Prince");
            assert_eq!(author.forename(), Some("Prince"));
            assert_eq!(author.surname(), None);
        }

        #[test]
        fn new_with_owned_strings() {
            // Ensure generics work with owned Strings
            let f = String::from("John");
            let l = String::from("Doe");
            let author = Author::new(f, l);
            assert_eq!(author.forename(), Some("John"));
            assert_eq!(author.surname(), Some("Doe"));
        }

        #[test]
        fn deserialized_empty_author_is_representable() {
            let author: Author = serde_json::from_str("{}").unwrap();
            assert_eq!(author.forename(), None);
            assert_eq!(author.surname(), None);
        }
    }

    mod instance_normalization {
        use super::*;

        #[test]
        fn forename_normalized() {
            let author = Author::new("  john  ", "Doe");
            let result = author.forename_normalized().unwrap();
            assert_eq!(result, Some(Cow::Borrowed("John")));
        }

        #[test]
        fn forename_none() {
            let author = Author::surname_only("Doe");
            let result = author.forename_normalized().unwrap();
            assert_eq!(result, None);
        }

        #[test]
        fn surname_normalized_standard() {
            let author = Author::new("John", "  doe  ");
            let result = author.surname_normalized().unwrap();
            assert_eq!(result.as_deref(), Some("Doe"));
            assert!(matches!(result, Some(Cow::Owned(_))));
        }

        #[test]
        fn surname_none_returns_none() {
            let author = Author::forename_only("Prince");
            let result = author.surname_normalized().unwrap();
            assert_eq!(result, None);
        }

        #[test]
        fn mac_expansion_survives_untrimmed_input() {
            let author = Author::new("Ronald", "  mcdonald  ");
            let result = author.surname_normalized().unwrap();
            assert_eq!(result.as_deref(), Some("Macdonald"));
        }
    }

    mod static_composed {
        use super::*;

        #[test]
        fn standard_composition() {
            let res = Author::composed_normalized("john", "doe").unwrap();
            assert_eq!(res, "Doe, John");
        }

        #[test]
        fn trimming_and_casing() {
            let res = Author::composed_normalized("  ALICE  ", "  wONDERLAND ").unwrap();
            assert_eq!(res, "Wonderland, Alice");
        }

        #[test]
        fn saint_formatting() {
            let res = Author::composed_normalized("Paul", "Saint").unwrap();
            assert_eq!(res, "Saint Paul");
        }

        #[test]
        fn sainte_formatting() {
            let res = Author::composed_normalized("Beuve", "Sainte").unwrap();
            assert_eq!(res, "Sainte Beuve");
        }

        #[test]
        fn san_formatting() {
            let res = Author::composed_normalized("Francisco", "San").unwrap();
            assert_eq!(res, "San Francisco");
        }

        #[test]
        fn st_expansion() {
            let res = Author::composed_normalized("Patrick", "st").unwrap();
            assert_eq!(res, "Saint Patrick");
        }

        #[test]
        fn ste_expansion() {
            let res = Author::composed_normalized("Marie", "Ste.").unwrap();
            assert_eq!(res, "Sainte Marie");
        }

        #[test]
        fn composed_with_bad_input() {
            let res = Author::composed_normalized("", "Doe");
            assert!(matches!(res, Err(NormalizationError::EmptyString)));
        }
    }
}
