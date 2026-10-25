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
        self.stripped_title().unwrap_or(&self.title)
    }

    /// The title without its initial article, or `None` when no article
    /// language is set or nothing was stripped.
    fn stripped_title(&self) -> Option<&str> {
        let language = self.article_language?;
        language.strip_initial_article(&self.title)
    }

    /// Runs `compute` on the [filing title](Self::filing_title), under the
    /// rule that stripping never turns a title that works into an error: when
    /// an article was stripped and `compute` fails with
    /// [`NormalizationError::EmptyString`], it runs again on the title as
    /// written. Any other result is returned as is. Every generator reads a
    /// title through it.
    ///
    /// Only the Cutter-Sanborn computations can take the fallback: they file
    /// on letters, so _The 100_ under English, filed on `100`, gives them
    /// nothing and they use _The 100_. `LoC` files `100` by its numeral rule,
    /// and a filing title starts with a letter or an ASCII digit, so `LoC`
    /// never fails with `EmptyString` there.
    ///
    /// `compute` may run twice, so it should be free of side effects; the
    /// `Fn` bound documents that, but doesn't enforce it.
    pub(crate) fn compute_on_filing_title<'a, T>(
        &'a self,
        compute: impl Fn(&'a str) -> Result<T, NormalizationError>,
    ) -> Result<T, NormalizationError> {
        let Some(filing) = self.stripped_title() else {
            return compute(&self.title);
        };
        let result = compute(filing);
        if matches!(result, Err(NormalizationError::EmptyString)) {
            compute(&self.title)
        } else {
            result
        }
    }

    /// Normalizes the [filing title](Self::filing_title) and truncates it to
    /// at most `suffix_len` characters, producing the lowercase-able work mark
    /// material used by
    /// [Cutter-Sanborn generation](crate::catalog::cutter::CutterSanbornMethod).
    /// When the filing title has no letters (_The 100_ under English), the
    /// title as written is used instead.
    ///
    /// The result is guaranteed to be ASCII-alphabetic. It may be shorter
    /// than `suffix_len` (including empty when `suffix_len == 0`).
    ///
    /// # Errors
    /// Returns a [`NormalizationError`] if the title normalizes to an empty
    /// string or contains untransliterable non-ASCII characters.
    pub fn as_suffix(&self, suffix_len: usize) -> Result<Cow<'_, str>, NormalizationError> {
        let cow =
            self.compute_on_filing_title(|title| to_ascii_alphabetic(Cow::Borrowed(title)))?;

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
    use std::cell::RefCell;

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

    /// Asserts that `compute_on_filing_title`, with a normalizing `compute`,
    /// returns `want` (an error as its message) after calling `compute` with
    /// `calls`.
    #[track_caller]
    fn assert_filed(
        title: &str,
        language: Option<Language>,
        want: Result<&str, &str>,
        calls: &[&str],
    ) {
        let opus = Opus::new(title).with_article_language(language);
        let seen = RefCell::new(Vec::new());
        let result = opus
            .compute_on_filing_title(|filing| {
                seen.borrow_mut().push(filing.to_owned());
                to_ascii_alphabetic(Cow::Borrowed(filing)).map(Cow::into_owned)
            })
            .map_err(|error| error.to_string());
        assert_eq!(result.as_deref().map_err(String::as_str), want, "{title}");
        assert_eq!(seen.into_inner(), calls, "{title}");
    }

    #[test]
    fn compute_on_filing_title_falls_back_only_when_stripping_empties_it() {
        let english = Some(Language::English);
        let empty = NormalizationError::EmptyString.to_string();
        let non_ascii = NormalizationError::NonAsciiAlpha("Война".to_owned()).to_string();
        assert_filed("The Bear", english, Ok("Bear"), &["Bear"]);
        // `100` has no letters, so the title as written is used.
        assert_filed("The 100", english, Ok("The"), &["100", "The 100"]);
        // Any other error propagates, without a second try.
        assert_filed("The Война", english, Err(&non_ascii), &["Война"]);
        // Nothing stripped: one try, on the title as written.
        assert_filed("The 100", None, Ok("The"), &["The 100"]);
        assert_filed("100", english, Err(&empty), &["100"]);
    }

    #[test]
    fn suffix_disregards_the_initial_article() {
        let english = Some(Language::English);
        let opus = Opus::new("A Christmas Carol").with_article_language(english);
        assert_eq!(opus.as_suffix(3).unwrap(), "Chr");
        // A clean filing title is still borrowed, not copied.
        let bear = Opus::new("The Bear").with_article_language(english);
        assert!(matches!(bear.as_suffix(2).unwrap(), Cow::Borrowed("Be")));
        assert_eq!(Opus::new("A Christmas Carol").as_suffix(3).unwrap(), "ACh");
        // The fallback: `The 100` gives today's mark.
        let numeral = Opus::new("The 100").with_article_language(english);
        assert_eq!(numeral.as_suffix(3).unwrap(), "The");
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
