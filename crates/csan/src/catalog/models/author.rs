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
/// constructible through [`Author::new`], [`Author::family_only`], or
/// [`Author::given_only`], but can arise through deserialization.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Author {
    /// The author's given (first) name, e.g. `"William"`.
    #[serde(default)]
    given_name: Option<String>,
    /// The author's family (last) name, e.g. `"Shakespeare"`.
    #[serde(default)]
    family_name: Option<String>,
}

impl Author {
    /// Creates a new `Author` with both a given name and a family name.
    ///
    /// # Examples
    /// ```
    /// # use csan::catalog::models::author::Author;
    /// let author = Author::new("William", "Shakespeare");
    /// ```
    #[must_use]
    pub fn new(given_name: impl Into<String>, family_name: impl Into<String>) -> Self {
        Self {
            given_name: Some(given_name.into()),
            family_name: Some(family_name.into()),
        }
    }

    /// Creates an `Author` with only a family name.
    #[must_use]
    pub fn family_only(family_name: impl Into<String>) -> Self {
        Self {
            given_name: None,
            family_name: Some(family_name.into()),
        }
    }

    /// Creates an `Author` with only a given name.
    #[must_use]
    pub fn given_only(given_name: impl Into<String>) -> Self {
        Self {
            given_name: Some(given_name.into()),
            family_name: None,
        }
    }

    /// Returns the author's raw (unnormalized) given name, if present.
    #[inline]
    #[must_use]
    pub fn given_name(&self) -> Option<&str> {
        self.given_name.as_deref()
    }

    /// Returns the author's raw (unnormalized) family name, if present.
    #[inline]
    #[must_use]
    pub fn family_name(&self) -> Option<&str> {
        self.family_name.as_deref()
    }

    /// Retrieves the given name and applies normalization.
    ///
    /// Returns `Ok(None)` if the author does not have a given name.
    ///
    /// # Errors
    /// Returns a [`NormalizationError`] if a present name fails
    /// [`Author::normalize_name`].
    pub fn given_name_normalized(&self) -> Result<Option<Cow<'_, str>>, NormalizationError> {
        self.given_name().map(Author::normalize_name).transpose()
    }

    /// Retrieves the family name and applies normalization.
    ///
    /// Returns `Ok(None)` if the author does not have a family name.
    ///
    /// # Errors
    /// Returns a [`NormalizationError`] if a present name fails
    /// [`Author::normalize_name`].
    pub fn family_name_normalized(&self) -> Result<Option<Cow<'_, str>>, NormalizationError> {
        self.family_name().map(Author::normalize_name).transpose()
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
    pub(crate) fn compose_parts(family_normalized: &str, given_normalized: &str) -> String {
        match family_normalized {
            "Saint" | "Sainte" | "San" => format!("{family_normalized} {given_normalized}"),
            "St" => format!("Saint {given_normalized}"),
            "Ste" => format!("Sainte {given_normalized}"),
            _ => format!("{family_normalized}, {given_normalized}"),
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
            assert_eq!(author.given_name(), Some("John"));
            assert_eq!(author.family_name(), Some("Doe"));
        }

        #[test]
        fn new_with_no_given_name() {
            let author = Author::family_only("Plato");
            assert_eq!(author.given_name(), None);
            assert_eq!(author.family_name(), Some("Plato"));
        }

        #[test]
        fn new_with_no_family_name() {
            let author = Author::given_only("Prince");
            assert_eq!(author.given_name(), Some("Prince"));
            assert_eq!(author.family_name(), None);
        }

        #[test]
        fn new_with_owned_strings() {
            // Ensure generics work with owned Strings
            let f = String::from("John");
            let l = String::from("Doe");
            let author = Author::new(f, l);
            assert_eq!(author.given_name(), Some("John"));
            assert_eq!(author.family_name(), Some("Doe"));
        }

        #[test]
        fn deserialized_empty_author_is_representable() {
            let author: Author = serde_json::from_str("{}").unwrap();
            assert_eq!(author.given_name(), None);
            assert_eq!(author.family_name(), None);
        }
    }

    mod instance_normalization {
        use super::*;

        #[test]
        fn given_name_normalized() {
            let author = Author::new("  john  ", "Doe");
            let result = author.given_name_normalized().unwrap();
            assert_eq!(result, Some(Cow::Borrowed("John")));
        }

        #[test]
        fn given_name_none() {
            let author = Author::family_only("Doe");
            let result = author.given_name_normalized().unwrap();
            assert_eq!(result, None);
        }

        #[test]
        fn family_name_normalized_standard() {
            let author = Author::new("John", "  doe  ");
            let result = author.family_name_normalized().unwrap();
            assert_eq!(result.as_deref(), Some("Doe"));
            assert!(matches!(result, Some(Cow::Owned(_))));
        }

        #[test]
        fn family_name_none_returns_none() {
            let author = Author::given_only("Prince");
            let result = author.family_name_normalized().unwrap();
            assert_eq!(result, None);
        }

        #[test]
        fn mac_expansion_survives_untrimmed_input() {
            let author = Author::new("Ronald", "  mcdonald  ");
            let result = author.family_name_normalized().unwrap();
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
