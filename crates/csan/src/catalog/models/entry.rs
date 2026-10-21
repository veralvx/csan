//! Defines the [`Attribution`] model.

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
    /// # use csan::catalog::models::entry::Attribution;
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
        given_name: Option<&str>,
        family_name: Option<&str>,
        title: Option<&str>,
    ) -> Option<Self> {
        let given = given_name.map(str::trim).filter(|n| !n.is_empty());
        let family = family_name.map(str::trim).filter(|n| !n.is_empty());
        let author = match (given, family) {
            (Some(g), Some(f)) => Some(Author::new(g, f)),
            (None, Some(f)) => Some(Author::family_only(f)),
            (Some(g), None) => Some(Author::given_only(g)),
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
        assert_eq!(author.given_name(), Some("Prince"));
        assert_eq!(author.family_name(), None);

        assert!(Attribution::from_parts(Some(""), Some("  "), Some("\t")).is_none());
    }

    #[test]
    fn parts_are_trimmed() {
        let Some(Attribution::Both { author, opus }) =
            Attribution::from_parts(Some("  Charles "), Some(" Dickens"), Some(" Oliver "))
        else {
            panic!("expected a both attribution");
        };
        assert_eq!(author.given_name(), Some("Charles"));
        assert_eq!(author.family_name(), Some("Dickens"));
        assert_eq!(opus.title(), "Oliver");
    }
}
