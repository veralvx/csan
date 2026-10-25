//! Defines the [`Opus`] model.
use crate::catalog::language::Language;
use crate::catalog::normalization::{NormalizationError, to_ascii_alphabetic};
use std::borrow::Cow;

/// Represents a work (book, article, etc.) identified by its title.
#[derive(Debug, Clone)]
pub struct Opus {
    /// The raw (unnormalized) title of the work.
    title: String,
    /// The language whose initial article the title is filed without, if any.
    article_language: Option<Language>,
}

impl Opus {
    /// Creates a new `Opus` from a title, filed as written (no article
    /// language).
    #[must_use]
    pub fn new(title: impl Into<String>) -> Opus {
        Opus {
            title: title.into(),
            article_language: None,
        }
    }

    /// Sets the language whose initial article the title is filed without
    /// (see [`Language::strip_initial_article`]); `None` files the title as
    /// written. The last call wins.
    ///
    /// # Examples
    /// ```
    /// use csan_core::{Language, Opus};
    ///
    /// let opus = Opus::new("A Christmas Carol").with_article_language(Some(Language::English));
    /// assert_eq!(opus.article_language(), Some(Language::English));
    /// assert_eq!(opus.filing_title(), "Christmas Carol");
    /// assert_eq!(opus.title(), "A Christmas Carol");
    ///
    /// let opus = opus.with_article_language(None);
    /// assert_eq!(opus.filing_title(), "A Christmas Carol");
    /// ```
    #[must_use]
    pub fn with_article_language(self, language: Option<Language>) -> Self {
        Opus {
            article_language: language,
            ..self
        }
    }

    /// The language whose initial article the title is filed without, if
    /// any.
    #[must_use]
    pub fn article_language(&self) -> Option<Language> {
        self.article_language
    }

    /// The title as filed: without its initial article when an article
    /// language is set and [`Language::strip_initial_article`] finds one, and
    /// otherwise the title as written. It is always a subslice of
    /// [`title`](Self::title).
    ///
    /// # Examples
    /// ```
    /// use csan_core::{Language, Opus};
    ///
    /// let english = Some(Language::English);
    /// assert_eq!(Opus::new("The Bear").with_article_language(english).filing_title(), "Bear");
    /// assert_eq!(Opus::new("Theory").with_article_language(english).filing_title(), "Theory");
    /// assert_eq!(Opus::new("The").with_article_language(english).filing_title(), "The");
    /// assert_eq!(Opus::new("The Bear").filing_title(), "The Bear");
    /// ```
    #[must_use]
    pub fn filing_title(&self) -> &str {
        self.article_language
            .and_then(|language| language.strip_initial_article(&self.title))
            .unwrap_or(&self.title)
    }

    /// Normalizes the title and truncates it to at most `suffix_len`
    /// characters, producing the lowercase-able work mark material used by
    /// [Cutter-Sanborn generation](crate::catalog::cutter::CutterSanbornMethod).
    ///
    /// The result is guaranteed to be ASCII-alphabetic. It may be shorter
    /// than `suffix_len` (including empty when `suffix_len == 0`).
    ///
    /// # Errors
    /// Returns a [`NormalizationError`] if the title normalizes to an empty
    /// string or contains untransliterable non-ASCII characters.
    pub fn as_suffix(&self, suffix_len: usize) -> Result<Cow<'_, str>, NormalizationError> {
        let cow = to_ascii_alphabetic(Cow::Borrowed(self.title()))?;

        // The normalized title is pure ASCII, so every byte index is a char
        // boundary and byte length equals char count.
        let limit = cow.len().min(suffix_len);

        match cow {
            Cow::Borrowed(s) => Ok(Cow::Borrowed(s.get(..limit).unwrap_or_default())),
            Cow::Owned(mut s) => {
                s.truncate(limit);
                Ok(Cow::Owned(s))
            }
        }
    }

    /// Returns the raw (unnormalized) title of the work.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suffix_truncates_and_normalizes() {
        let opus = Opus::new("  Öliver Twist!  ");
        assert_eq!(opus.as_suffix(1).unwrap(), "O");
        assert_eq!(opus.as_suffix(3).unwrap(), "Oli");
        assert_eq!(opus.as_suffix(100).unwrap(), "OliverTwist");
    }

    #[test]
    fn suffix_zero_len_is_empty() {
        let opus = Opus::new("Dune");
        assert_eq!(opus.as_suffix(0).unwrap(), "");
    }

    #[test]
    fn borrowed_input_stays_borrowed() {
        let opus = Opus::new("Dune");
        let suffix = opus.as_suffix(2).unwrap();
        assert_eq!(suffix, "Du");
        assert!(matches!(suffix, Cow::Borrowed(_)));
    }

    #[test]
    fn filing_title_follows_the_article_language() {
        let opus = Opus::new("The Bear");
        assert_eq!(opus.article_language(), None);
        assert_eq!(opus.filing_title(), "The Bear");

        let opus = opus.with_article_language(Some(Language::English));
        assert_eq!(opus.article_language(), Some(Language::English));
        assert_eq!(opus.filing_title(), "Bear");
        assert_eq!(opus.title(), "The Bear");

        // Another language's article isn't one; Latin has none at all.
        let french = opus.clone().with_article_language(Some(Language::French));
        assert_eq!(french.filing_title(), "The Bear");
        let latin = opus.clone().with_article_language(Some(Language::Latin));
        assert_eq!(latin.filing_title(), "The Bear");

        // The last call wins, and None turns stripping off.
        let off = opus.with_article_language(None);
        assert_eq!(off.article_language(), None);
        assert_eq!(off.filing_title(), "The Bear");
    }

    #[test]
    fn filing_title_is_a_subslice_of_the_title() {
        for (title, want) in [("The Bear", "Bear"), ("  The \"Bear\"  ", "Bear\"  ")] {
            let opus = Opus::new(title).with_article_language(Some(Language::English));
            assert_eq!(opus.filing_title(), want);
            let range = opus.title().as_bytes().as_ptr_range();
            let filing = opus.filing_title().as_bytes().as_ptr_range();
            assert!(
                range.start <= filing.start && filing.end <= range.end,
                "{title:?}"
            );
        }
    }

    #[test]
    fn empty_title_errors() {
        let opus = Opus::new("  !!! ");
        assert!(matches!(
            opus.as_suffix(3),
            Err(NormalizationError::EmptyString)
        ));
    }
}
