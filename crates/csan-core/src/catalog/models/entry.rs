//! Defines the [`Attribution`] model.

use crate::catalog::language::Language;
use crate::catalog::models::{author::Author, opus::Opus};

/// The bibliographic information a Cutter number is generated from.
///
/// Cutter numbers are normally derived from the author's name, but works
/// without a (usable) author are cuttered by title instead, and works with
/// both can carry a title-derived work mark. `Attribution` captures which of
/// these inputs are available for an entry.
///
/// Prefer [`Attribution::from_parts`] to construct one from optional inputs.
/// The type is `#[non_exhaustive]` so the crate can add input shapes in a
/// future release without a breaking change; downstream `match`es on it
/// therefore need a wildcard arm.
#[non_exhaustive]
pub enum Attribution {
    /// Both an author and a work are known.
    Both {
        /// The author of the work.
        author: Author,
        /// The work itself, used for the work mark.
        opus: Opus,
    },
    /// Only the author is known.
    AuthorOnly(Author),
    /// Only the work is known; the Cutter number is derived from its title.
    OpusOnly(Opus),
}

impl Attribution {
    /// Builds an `Attribution` from optional name and title parts, treating
    /// blank (empty or whitespace-only) strings as absent.
    ///
    /// This is the shared front door for user-facing inputs (CLI arguments,
    /// plugin argument maps, FFI bindings): each part is trimmed, blank
    /// parts are dropped, and `None` is returned when nothing usable
    /// remains.
    ///
    /// # Examples
    /// ```
    /// # use csan_core::catalog::models::entry::Attribution;
    /// assert!(matches!(
    ///     Attribution::from_parts(Some("Charles"), Some("Dickens"), None),
    ///     Some(Attribution::AuthorOnly(_))
    /// ));
    /// assert!(matches!(
    ///     Attribution::from_parts(None, Some("  "), Some("Dune")),
    ///     Some(Attribution::OpusOnly(_))
    /// ));
    /// assert!(Attribution::from_parts(None, Some("  "), None).is_none());
    /// ```
    #[must_use]
    pub fn from_parts(
        forename: Option<&str>,
        surname: Option<&str>,
        title: Option<&str>,
    ) -> Option<Self> {
        let given = forename.map(str::trim).filter(|n| !n.is_empty());
        let family = surname.map(str::trim).filter(|n| !n.is_empty());
        let author = match (given, family) {
            (Some(g), Some(f)) => Some(Author::new(g, f)),
            (None, Some(f)) => Some(Author::surname_only(f)),
            (Some(g), None) => Some(Author::forename_only(g)),
            (None, None) => None,
        };
        let opus = title
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .map(Opus::new);

        match (author, opus) {
            (Some(author), Some(opus)) => Some(Attribution::Both { author, opus }),
            (Some(author), None) => Some(Attribution::AuthorOnly(author)),
            (None, Some(opus)) => Some(Attribution::OpusOnly(opus)),
            (None, None) => None,
        }
    }

    /// Sets the language whose initial article the entry's title is filed
    /// without, as [`Opus::with_article_language`] does; `None` files the
    /// title as written. It applies to the title of `Both` and `OpusOnly`, and
    /// does nothing for `AuthorOnly`. Author names are never stripped: an
    /// article in a name is part of the heading (LC G 100 §10).
    ///
    /// Frontends call it after [`from_parts`](Self::from_parts).
    ///
    /// # Examples
    /// ```
    /// use csan_core::{Attribution, Language};
    ///
    /// let entry = Attribution::from_parts(None, Some("Dickens"), Some("A Christmas Carol"))
    ///     .map(|entry| entry.with_article_language(Some(Language::English)));
    /// assert!(matches!(
    ///     entry,
    ///     Some(Attribution::Both { ref opus, .. }) if opus.filing_title() == "Christmas Carol"
    /// ));
    /// ```
    #[must_use]
    pub fn with_article_language(self, language: Option<Language>) -> Self {
        match self {
            Attribution::Both { author, opus } => Attribution::Both {
                author,
                opus: opus.with_article_language(language),
            },
            Attribution::OpusOnly(opus) => {
                Attribution::OpusOnly(opus.with_article_language(language))
            }
            Attribution::AuthorOnly(author) => Attribution::AuthorOnly(author),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_parts_selects_the_right_variant() {
        assert!(matches!(
            Attribution::from_parts(Some("Charles"), Some("Dickens"), Some("Oliver Twist")),
            Some(Attribution::Both { .. })
        ));
        assert!(matches!(
            Attribution::from_parts(None, Some("Dickens"), None),
            Some(Attribution::AuthorOnly(_))
        ));
        assert!(matches!(
            Attribution::from_parts(None, None, Some("Bear")),
            Some(Attribution::OpusOnly(_))
        ));
        assert!(Attribution::from_parts(None, None, None).is_none());
    }

    #[test]
    fn blank_parts_are_absent() {
        // A blank field must not poison a usable one.
        let Some(Attribution::AuthorOnly(author)) =
            Attribution::from_parts(Some("Prince"), Some("   "), None)
        else {
            panic!("expected an author-only attribution");
        };
        assert_eq!(author.forename(), Some("Prince"));
        assert_eq!(author.surname(), None);

        assert!(Attribution::from_parts(Some(""), Some("  "), Some("\t")).is_none());
    }

    #[test]
    fn parts_are_trimmed() {
        let Some(Attribution::Both { author, opus }) =
            Attribution::from_parts(Some("  Charles "), Some(" Dickens"), Some(" Oliver "))
        else {
            panic!("expected a both attribution");
        };
        assert_eq!(author.forename(), Some("Charles"));
        assert_eq!(author.surname(), Some("Dickens"));
        assert_eq!(opus.title(), "Oliver");
    }

    /// The article language of the entry's title, or `None` for `AuthorOnly`.
    fn article_language(entry: &Attribution) -> Option<Language> {
        match entry {
            Attribution::Both { opus, .. } | Attribution::OpusOnly(opus) => opus.article_language(),
            Attribution::AuthorOnly(_) => None,
        }
    }

    #[test]
    fn article_language_reaches_the_title() {
        let english = Some(Language::English);
        for entry in [
            Attribution::from_parts(None, Some("Dickens"), Some("A Christmas Carol")),
            Attribution::from_parts(None, None, Some("A Christmas Carol")),
        ] {
            let entry = entry.map(|entry| entry.with_article_language(english));
            let Some(Attribution::Both { opus, .. } | Attribution::OpusOnly(opus)) = entry else {
                panic!("expected a title");
            };
            assert_eq!(opus.article_language(), english);
            assert_eq!(opus.filing_title(), "Christmas Carol");
        }
    }

    #[test]
    fn last_article_language_wins() {
        for entry in [
            Attribution::Both {
                author: Author::surname_only("Faulkner"),
                opus: Opus::new("The Bear"),
            },
            Attribution::OpusOnly(Opus::new("The Bear")),
        ] {
            let entry = entry
                .with_article_language(Some(Language::French))
                .with_article_language(Some(Language::English));
            assert_eq!(article_language(&entry), Some(Language::English));
            let entry = entry.with_article_language(None);
            assert_eq!(article_language(&entry), None);
        }
    }

    #[test]
    fn author_names_are_never_stripped() {
        // G 100 §10: an article in a name is part of the heading.
        let entry =
            Attribution::from_parts(Some("Jean de"), Some("La Fontaine"), Some("Les Fables"))
                .map(|entry| entry.with_article_language(Some(Language::French)));
        let Some(Attribution::Both { author, opus }) = entry else {
            panic!("expected a both attribution");
        };
        assert_eq!(author.forename(), Some("Jean de"));
        assert_eq!(author.surname(), Some("La Fontaine"));
        assert_eq!(opus.filing_title(), "Fables");

        let author_only = Attribution::AuthorOnly(Author::surname_only("La Fontaine"))
            .with_article_language(Some(Language::French));
        let Attribution::AuthorOnly(kept) = author_only else {
            panic!("expected an author-only attribution");
        };
        assert_eq!(kept.surname(), Some("La Fontaine"));
    }
}
