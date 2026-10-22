//! Defines the [`Opus`] model.
use crate::catalog::normalization::{NormalizationError, to_ascii_alphabetic};
use std::borrow::Cow;

/// Represents a work (book, article, etc.) identified by its title.
#[derive(Debug, Clone)]
pub struct Opus {
    /// The raw (unnormalized) title of the work.
    title: String,
}

impl Opus {
    /// Creates a new `Opus` from a title.
    #[must_use]
    pub fn new(title: impl Into<String>) -> Opus {
        Opus {
            title: title.into(),
        }
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
    fn empty_title_errors() {
        let opus = Opus::new("  !!! ");
        assert!(matches!(
            opus.as_suffix(3),
            Err(NormalizationError::EmptyString)
        ));
    }
}
