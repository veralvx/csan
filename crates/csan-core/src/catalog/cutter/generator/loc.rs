//! The Library of Congress (`LoC`) Cutter generation method and its digit
//! rules.

use std::borrow::Cow;
use std::iter::Peekable;
use std::str::Chars;

use crate::catalog::cutter::tables::CutterNumType;
use crate::catalog::models::{cutter::CutterLoc, entry::Attribution};
use crate::catalog::normalization::{NormalizationError, capitalize_ascii, to_ascii_alphabetic};

use super::common::{CutterConfig, CutterGenerator};

/// A marker struct for the [Library of Congress
/// (`LoC`)](https://www.loc.gov/aba/pcc/053/table.html) Cutter generation
/// method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CutterLocMethod;

/// Digit assigned after an initial vowel (`LoC` table, "After initial
/// vowels").
fn vowel_digit(c: char) -> CutterNumType {
    match c {
        'a'..='c' => 2,
        'd'..='k' => 3,
        'l' | 'm' => 4,
        'n' | 'o' => 5,
        'p' | 'q' => 6,
        'r' => 7,
        's' | 't' => 8,
        _ => 9,
    }
}

/// Digit assigned after an initial consonant (`LoC` table, "After other
/// initial consonants").
fn consonant_digit(c: char) -> CutterNumType {
    match c {
        'a'..='d' => 3,
        'e'..='h' => 4,
        'i'..='n' => 5,
        'o'..='q' => 6,
        'r'..='t' => 7,
        'u'..='x' => 8,
        _ => 9,
    }
}

/// Digit assigned after the initial letter `S` (`LoC` table, "After initial
/// letter S").
///
/// The `LoC`/PCC table lists only breakpoints; per the standard reading
/// (Yale SCM: Shelflisting: "a single letter at the head of a column implies
/// a range of letters up to the next column"), each breakpoint covers the
/// letters from itself up to the next. Because the standalone letter `c`
/// sorts before the digraph `ch`, a plain `Sc` falls in the `a` range (`2`);
/// only the `ch` digraph itself takes `3`, consuming the `h`. This matches
/// the `LoC`/Yale worked examples Sadron→S23, Schreiber→S37, Stinson→S75.
fn s_digit(c: char, rest: &mut Peekable<Chars<'_>>) -> CutterNumType {
    match c {
        // `a` range: a, b, and standalone c (c < "ch") -> 2.
        'a' | 'b' => 2,
        'c' => match rest.peek() {
            // The `ch` digraph is the only `Sc-` taking 3; consume the `h`.
            Some(&'h') => {
                rest.next();
                3
            }
            _ => 2,
        },
        // `ch` range continues through d -> 3.
        'd' => 3,
        'e'..='g' => 4,
        'h'..='l' => 5,
        'm'..='s' => 6,
        't' => 7,
        // `u` range: u and v (v < "w") -> 8.
        'u' | 'v' => 8,
        // `w-z` range -> 9.
        _ => 9,
    }
}

/// Digits assigned after the initial letters `Qu`/`Qa`–`Qt` (`LoC` table,
/// "After the initial letters Qu").
///
/// The `Qu` third-letter values follow the `LoC` table exactly (with `Qut`
/// kept at 8). For `Qa`–`Qt` the `LoC` table specifies only "use 2-29" — a
/// range to distribute within based on the local shelflist, with no
/// per-letter formula — so the monotonic spread over `2`–`29` returned here
/// is one valid, deterministic distribution rather than a fixed `LoC` value.
/// Two-digit assignments are returned as a digit pair.
fn q_digits(c: char, rest: &mut Peekable<Chars<'_>>) -> (CutterNumType, Option<CutterNumType>) {
    match c {
        'u' => match rest.next() {
            Some('t') => (8, None),
            Some(letter) => (consonant_digit(letter), None),
            None => (3, None),
        },
        'a' => (2, None),
        'b' | 'c' => (4, None),
        'd' | 'e' => (6, None),
        'f' | 'g' => (8, None),
        'h' | 'i' => (1, Some(1)),
        'j'..='l' => (1, Some(3)),
        'm' | 'n' => (1, Some(5)),
        'o' | 'p' => (1, Some(7)),
        'q' | 'r' => (2, Some(1)),
        's' => (2, Some(4)),
        't' => (2, Some(7)),
        _ => (consonant_digit(c), None),
    }
}

/// Digits assigned by the initial-letter rule, dispatched on the first
/// letter of the name.
fn initial_digits(
    first_char: char,
    c: char,
    rest: &mut Peekable<Chars<'_>>,
) -> (CutterNumType, Option<CutterNumType>) {
    match first_char {
        'A' | 'E' | 'I' | 'O' | 'U' => (vowel_digit(c), None),
        'S' => (s_digit(c, rest), None),
        'Q' => q_digits(c, rest),
        _ => (consonant_digit(c), None),
    }
}

/// Digit assigned by the expansion rule (`LoC` table, "For expansion").
///
/// The input is ASCII-alphabetic by construction; anything else degrades to
/// the top bucket instead of panicking.
fn expansion_digit(c: char) -> CutterNumType {
    match c.to_ascii_lowercase() {
        'a'..='d' => 3,
        'e'..='h' => 4,
        'i'..='l' => 5,
        'm'..='o' => 6,
        'p'..='s' => 7,
        't'..='v' => 8,
        _ => 9,
    }
}

/// The expansion material for a title that starts with a numeral: the text
/// after the first whitespace, or — for a single compact token like
/// `"100best"` — the token with its leading digits stripped.
fn numeral_remainder(raw_trimmed: &str) -> &str {
    match raw_trimmed.split_once(char::is_whitespace) {
        Some((_, rest)) => rest,
        None => raw_trimmed.trim_start_matches(|c: char| c.is_ascii_digit()),
    }
}

impl CutterLocMethod {
    /// The most digits a generated [`CutterLoc`] can carry.
    ///
    /// [`CutterConfig::max_len`] values above this are clamped down to it,
    /// which also guarantees the digit accumulation can never overflow
    /// [`CutterNumType`].
    pub const MAX_NUMERIC_LEN: usize = 4;
}

impl CutterGenerator for CutterLocMethod {
    type CutterType = CutterLoc;

    /// Follows the [Library of Congress (LoC)](https://www.loc.gov/aba/pcc/053/table.html) rules.
    /// For reference: [Classification and Shelflisting Manual Instruction
    /// Sheets, Sheet G 63 (Cutter
    /// Numbers)](https://www.loc.gov/aba/publications/FreeCSM/G063.pdf).
    ///
    /// If a name starts with a number, it must be assigned to the `A12`-`A19`
    /// range. The exact value can be arbitrarily chosen by the librarian, as
    /// can the following digits for expansion. This function defaults to
    /// `A15`, and further digits come from the "For expansion" section,
    /// matching the letters that follow the numeral. All other characters
    /// are normalized, and non-ASCII-alphabetic characters are ignored.
    /// Thus, if a name does not *start* with a number but *contains* one,
    /// the number is not converted to its spelled form (for the sake of
    /// being language agnostic); it is entirely ignored.
    ///
    /// If the name starts with `Qa`-`Qt`, it can be assigned to the `2`-`29`
    /// range, which is also arbitrary. This function assigns a number in
    /// this range, up to `27`, leaving gaps for the possibility of
    /// unclustering names starting with the same letters.
    ///
    /// `config.max_len` is clamped as documented on
    /// [`CutterConfig::max_len`].
    ///
    /// # Errors
    /// * [`NormalizationError::EmptyString`] if `config.max_len` is `0`, if
    ///   the entry's name normalizes to empty, or if an
    ///   [`Author`](crate::catalog::models::author::Author) carries
    ///   neither name.
    /// * [`NormalizationError::NonAsciiAlpha`] if the name contains
    ///   untransliterable characters.
    ///
    /// # Examples
    /// ```
    /// # use csan_core::catalog::models::author::Author;
    /// # use csan_core::catalog::models::opus::Opus;
    /// # use csan_core::catalog::models::cutter::CutterLoc;
    /// # use csan_core::catalog::models::entry::Attribution;
    /// # use csan_core::catalog::cutter::generator::{CutterGenerator, CutterLocMethod, CutterConfig};
    ///
    /// // Standard consonant rule: C -> a(3) -> r(7) -> t(8)
    /// let aut_attr = Attribution::AuthorOnly(Author::surname_only("Carter"));
    /// let result = CutterLocMethod::generate(&aut_attr, CutterConfig {max_len: 3, ..Default::default()}).unwrap();
    ///
    /// assert_eq!(result, CutterLoc::new('C', 378));
    ///
    /// // Numeral rule: assigns the A15 span, expands on 'b' in "best" (a-d = 3)
    /// let title_attr = Attribution::OpusOnly(Opus::new("100 best recipes"));
    /// let result_num = CutterLocMethod::generate(&title_attr, CutterConfig {max_len: 3, ..Default::default()}).unwrap();
    ///
    /// assert_eq!(result_num, CutterLoc::new('A', 153));
    /// ```
    fn generate(
        entry: &Attribution,
        config: CutterConfig,
    ) -> Result<CutterLoc, NormalizationError> {
        if config.max_len == 0 {
            return Err(NormalizationError::EmptyString);
        }

        let raw_str = match entry {
            Attribution::Both { author, .. } | Attribution::AuthorOnly(author) => author
                .surname()
                .or_else(|| author.forename())
                .ok_or(NormalizationError::EmptyString)?,
            Attribution::OpusOnly(opus) => opus.title(),
        };

        let raw_trimmed = raw_str.trim_start();

        let starts_with_number = raw_trimmed
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_digit());

        // The numeral rule's A15 span already occupies two digits.
        let max_len = if starts_with_number {
            config.max_len.clamp(2, Self::MAX_NUMERIC_LEN)
        } else {
            config.max_len.min(Self::MAX_NUMERIC_LEN)
        };

        let mut numbers: Vec<CutterNumType> = Vec::with_capacity(max_len);
        let mut apply_initial_rule = true;
        let first_char;
        let working_name;

        if starts_with_number {
            // Assign the LoC A12-A19 numeral span (defaults to A15).
            numbers.extend([1, 5]);
            first_char = 'A';
            apply_initial_rule = false;

            match to_ascii_alphabetic(Cow::Borrowed(numeral_remainder(raw_trimmed))) {
                Ok(rest) => working_name = capitalize_ascii(rest),
                // A bare number carries no expansion material.
                Err(NormalizationError::EmptyString) => return Ok(CutterLoc::new('A', 15)),
                Err(other) => return Err(other),
            }
        } else {
            working_name = capitalize_ascii(to_ascii_alphabetic(Cow::Borrowed(raw_trimmed))?);

            first_char = working_name
                .chars()
                .next()
                .ok_or(NormalizationError::EmptyString)?;
        }

        let mut name_iter = working_name.chars().peekable();

        if !starts_with_number {
            name_iter.next();
        }

        while let Some(c) = name_iter.next() {
            if numbers.len() >= max_len {
                break;
            }

            let (first_digit, second_digit) = if apply_initial_rule {
                apply_initial_rule = false;
                initial_digits(first_char, c, &mut name_iter)
            } else {
                (expansion_digit(c), None)
            };

            numbers.push(first_digit);
            if let Some(digit) = second_digit
                && numbers.len() < max_len
            {
                numbers.push(digit);
            }
        }

        if numbers.is_empty() {
            numbers.push(2);
        }

        // Bounded by construction: at most MAX_NUMERIC_LEN (4) digits of at
        // most 9 each, so the accumulator peaks at 9999 << CutterNumType::MAX.
        let numeric = numbers.into_iter().fold(0, |acc: CutterNumType, digit| {
            acc.saturating_mul(10).saturating_add(digit)
        });

        Ok(CutterLoc::new(first_char, numeric))
    }
}

#[cfg(test)]
mod tests {
    use crate::catalog::models::{Opus, author::Author};

    use super::*;

    /// Helper
    fn gen_author(name: &str, max_len: usize) -> Result<CutterLoc, NormalizationError> {
        let author = Author::surname_only(name);
        let attr = Attribution::AuthorOnly(author);
        CutterLocMethod::generate(
            &attr,
            CutterConfig {
                max_len,
                ..Default::default()
            },
        )
    }

    /// Helper
    fn gen_title(title: &str, max_len: usize) -> Result<CutterLoc, NormalizationError> {
        let entry = Attribution::OpusOnly(Opus::new(title));
        CutterLocMethod::generate(
            &entry,
            CutterConfig {
                max_len,
                ..Default::default()
            },
        )
    }

    #[test]
    fn errors_and_boundaries() {
        assert!(matches!(
            gen_author("Smith", 0),
            Err(NormalizationError::EmptyString)
        ));
        assert!(matches!(
            gen_author("   ", 3),
            Err(NormalizationError::EmptyString)
        ));
    }

    #[test]
    fn author_with_no_names_errors() {
        let author: Author = serde_json::from_str("{}").unwrap();
        assert!(matches!(
            CutterLocMethod::generate(&Attribution::AuthorOnly(author), CutterConfig::default()),
            Err(NormalizationError::EmptyString)
        ));
    }

    #[test]
    fn forename_only_author_uses_forename() {
        let attr = Attribution::AuthorOnly(Author::forename_only("Prince"));
        let result = CutterLocMethod::generate(&attr, CutterConfig::default()).unwrap();
        assert_eq!(result, CutterLoc::new('P', 756));
    }

    #[test]
    fn vowel_rules() {
        assert_eq!(gen_author("Adams", 3).unwrap(), CutterLoc::new('A', 336));
        assert_eq!(gen_author("Ibanez", 3).unwrap(), CutterLoc::new('I', 236));
        assert_eq!(gen_author("Updike", 3).unwrap(), CutterLoc::new('U', 635));
    }

    #[test]
    fn consonant_rules() {
        assert_eq!(gen_author("Carter", 3).unwrap(), CutterLoc::new('C', 378));
        assert_eq!(gen_author("Nguyen", 2).unwrap(), CutterLoc::new('N', 48));
    }

    #[test]
    fn s_rules() {
        assert_eq!(gen_author("Smith", 3).unwrap(), CutterLoc::new('S', 658));
        assert_eq!(gen_author("Schwartz", 3).unwrap(), CutterLoc::new('S', 393));
        // Per the LoC/Yale range rule, standalone `Sc` (c sorts before the
        // `ch` digraph) falls in the `a` range -> 2; only the `ch` digraph
        // itself takes 3. Matches Yale's worked example Schreiber -> S37.
        assert_eq!(gen_author("Sciolino", 1).unwrap(), CutterLoc::new('S', 2));
        assert_eq!(gen_author("Scott", 1).unwrap(), CutterLoc::new('S', 2));
        assert_eq!(gen_author("Scalia", 1).unwrap(), CutterLoc::new('S', 2));
    }

    /// Every worked example published on the LoC-following Yale Cutter
    /// table page, verified end to end through the real generator.
    #[test]
    fn loc_published_worked_examples() {
        let cases = [
            // (surname, expected two-significant-digit cutter)
            ("Idaho", 'I', 33),
            ("Inman", 'I', 56),
            ("Ipswich", 'I', 67),
            ("Sadron", 'S', 23),
            ("Schreiber", 'S', 37),
            ("Stinson", 'S', 75),
            ("Quade", 'Q', 33),
            ("Qutub", 'Q', 88),
            ("Campbell", 'C', 36),
            ("Ceccaldi", 'C', 43),
            ("Cryer", 'C', 79),
        ];
        for (name, alpha, numeric) in cases {
            assert_eq!(
                gen_author(name, 2).unwrap(),
                CutterLoc::new(alpha, numeric),
                "LoC/Yale published example {name} -> {alpha}{numeric}"
            );
        }
    }

    /// Exhaustive per-letter coverage of the vowel rule against the
    /// LoC/PCC table (loc.gov G 63) under LoC/Yale's range rule.
    #[test]
    fn vowel_rule_matches_loc_table_for_every_letter() {
        let vowel = [
            ('a', 2),
            ('b', 2),
            ('c', 2),
            ('d', 3),
            ('e', 3),
            ('f', 3),
            ('g', 3),
            ('h', 3),
            ('i', 3),
            ('j', 3),
            ('k', 3),
            ('l', 4),
            ('m', 4),
            ('n', 5),
            ('o', 5),
            ('p', 6),
            ('q', 6),
            ('r', 7),
            ('s', 8),
            ('t', 8),
            ('u', 9),
            ('v', 9),
            ('w', 9),
            ('x', 9),
            ('y', 9),
            ('z', 9),
        ];
        for (c, want) in vowel {
            assert_eq!(vowel_digit(c), want, "vowel rule, second letter {c}");
        }
    }

    /// Exhaustive per-letter coverage of the (non-S, non-Q) consonant rule.
    #[test]
    fn consonant_rule_matches_loc_table_for_every_letter() {
        let consonant = [
            ('a', 3),
            ('b', 3),
            ('c', 3),
            ('d', 3),
            ('e', 4),
            ('f', 4),
            ('g', 4),
            ('h', 4),
            ('i', 5),
            ('j', 5),
            ('k', 5),
            ('l', 5),
            ('m', 5),
            ('n', 5),
            ('o', 6),
            ('p', 6),
            ('q', 6),
            ('r', 7),
            ('s', 7),
            ('t', 7),
            ('u', 8),
            ('v', 8),
            ('w', 8),
            ('x', 8),
            ('y', 9),
            ('z', 9),
        ];
        for (c, want) in consonant {
            assert_eq!(
                consonant_digit(c),
                want,
                "consonant rule, second letter {c}"
            );
        }
    }

    /// Exhaustive per-letter coverage of the expansion rule.
    #[test]
    fn expansion_rule_matches_loc_table_for_every_letter() {
        let expansion = [
            ('a', 3),
            ('b', 3),
            ('c', 3),
            ('d', 3),
            ('e', 4),
            ('f', 4),
            ('g', 4),
            ('h', 4),
            ('i', 5),
            ('j', 5),
            ('k', 5),
            ('l', 5),
            ('m', 6),
            ('n', 6),
            ('o', 6),
            ('p', 7),
            ('q', 7),
            ('r', 7),
            ('s', 7),
            ('t', 8),
            ('u', 8),
            ('v', 8),
            ('w', 9),
            ('x', 9),
            ('y', 9),
            ('z', 9),
        ];
        for (c, want) in expansion {
            assert_eq!(expansion_digit(c), want, "expansion rule, letter {c}");
        }
    }

    /// Exhaustive per-letter coverage of the S rule, plus the `Sch`
    /// digraph exception and its `h` consumption.
    #[test]
    fn s_rule_matches_loc_table_for_every_letter() {
        // `c` is tested via a non-`h` follower so the digraph branch is
        // not taken; it is covered separately below.
        let s_single = [
            ('a', 2),
            ('b', 2),
            ('d', 3),
            ('e', 4),
            ('f', 4),
            ('g', 4),
            ('h', 5),
            ('i', 5),
            ('j', 5),
            ('k', 5),
            ('l', 5),
            ('m', 6),
            ('n', 6),
            ('o', 6),
            ('p', 6),
            ('q', 6),
            ('r', 6),
            ('s', 6),
            ('t', 7),
            ('u', 8),
            ('v', 8),
            ('w', 9),
            ('x', 9),
            ('y', 9),
            ('z', 9),
        ];
        for (c, want) in s_single {
            let mut follower = "x".chars().peekable(); // non-'h'
            assert_eq!(s_digit(c, &mut follower), want, "S rule, second letter {c}");
        }
        // Standalone `Sc` (no follower, and non-`h` follower) -> 2.
        assert_eq!(s_digit('c', &mut "".chars().peekable()), 2, "bare Sc");
        assert_eq!(s_digit('c', &mut "z".chars().peekable()), 2, "Sc + non-h");
        // The `ch` digraph -> 3, and it consumes the `h`.
        let mut ch = "hwartz".chars().peekable();
        assert_eq!(s_digit('c', &mut ch), 3, "Sch digraph");
        assert_eq!(ch.next(), Some('w'), "Sch consumes the h");
    }

    /// Exhaustive per-letter coverage of the `Qu` third-letter rule,
    /// including the `Qut` special case that keeps `t` at 8.
    #[test]
    fn qu_rule_matches_loc_table_for_every_third_letter() {
        let qu_third = [
            ('a', 3),
            ('b', 3),
            ('c', 3),
            ('d', 3),
            ('e', 4),
            ('f', 4),
            ('g', 4),
            ('h', 4),
            ('i', 5),
            ('j', 5),
            ('k', 5),
            ('l', 5),
            ('m', 5),
            ('n', 5),
            ('o', 6),
            ('p', 6),
            ('q', 6),
            ('r', 7),
            ('s', 7),
            ('t', 8),
            ('u', 8),
            ('v', 8),
            ('w', 8),
            ('x', 8),
            ('y', 9),
            ('z', 9),
        ];
        for (third, want) in qu_third {
            let follower = format!("{third}");
            let mut it = follower.chars().peekable();
            let (digit, second_digit) = q_digits('u', &mut it);
            assert_eq!(digit, want, "Qu rule, third letter {third}");
            assert_eq!(second_digit, None, "Qu third letter yields a single digit");
        }
    }

    #[test]
    fn q_rules() {
        assert_eq!(gen_author("Quigley", 2).unwrap(), CutterLoc::new('Q', 54));
        assert_eq!(gen_author("Qutb", 1).unwrap(), CutterLoc::new('Q', 8));
        assert_eq!(gen_author("Qadmon", 1).unwrap(), CutterLoc::new('Q', 2));
        assert_eq!(gen_author("Qhian", 2).unwrap(), CutterLoc::new('Q', 11));
    }

    #[test]
    fn multi_digit_q_truncation() {
        assert_eq!(gen_author("Qt...", 1).unwrap(), CutterLoc::new('Q', 2));
    }

    #[test]
    fn short_names() {
        assert_eq!(gen_author("A", 3).unwrap(), CutterLoc::new('A', 2));
        assert_eq!(gen_author("Qu", 3).unwrap(), CutterLoc::new('Q', 3));
    }

    #[test]
    fn case_insensitivity_in_peek_rules() {
        // `Sch` digraph detection and the `Qu` third-letter lookahead
        // both peek the (normalized-lowercase) next char; upper-case
        // input must resolve identically.
        assert_eq!(gen_author("SCHWARTZ", 3).unwrap(), CutterLoc::new('S', 393));
        assert_eq!(gen_author("SCIOLINO", 1).unwrap(), CutterLoc::new('S', 2));
        assert_eq!(gen_author("QUTB", 1).unwrap(), CutterLoc::new('Q', 8));
    }

    #[test]
    fn max_len_honored() {
        assert_eq!(gen_author("Macdonald", 1).unwrap(), CutterLoc::new('M', 3));
        assert_eq!(
            gen_author("Macdonald", 3).unwrap(),
            CutterLoc::new('M', 333)
        );
    }

    #[test]
    fn max_len_clamped_to_four() {
        assert_eq!(
            gen_author("Macdonald", 4).unwrap(),
            CutterLoc::new('M', 3336)
        );
        assert_eq!(
            gen_author("Macdonald", 64).unwrap(),
            gen_author("Macdonald", 4).unwrap(),
            "max_len above MAX_NUMERIC_LEN is clamped"
        );
    }

    #[test]
    fn title_entry() {
        assert_eq!(gen_title("Bear", 3).unwrap(), CutterLoc::new('B', 437));
    }

    #[test]
    fn loc_numeral_rules() {
        assert_eq!(
            gen_title("100 best hamburger recipes", 3).unwrap(),
            CutterLoc::new('A', 153)
        );

        assert_eq!(
            gen_title("1981 commercial composite infrared index", 4).unwrap(),
            CutterLoc::new('A', 1536)
        );

        assert_eq!(
            gen_title("1880's American arms", 4).unwrap(),
            CutterLoc::new('A', 1536)
        );

        assert_eq!(gen_title("100", 2).unwrap(), CutterLoc::new('A', 15));
        assert_eq!(gen_title("100 best", 2).unwrap(), CutterLoc::new('A', 15));
    }

    #[test]
    fn numeral_titles_never_lose_their_span() {
        // Even max_len == 1 keeps the two digits of the A15 span.
        assert_eq!(gen_title("100 best", 1).unwrap(), CutterLoc::new('A', 15));
        assert_eq!(gen_title("100", 1).unwrap(), CutterLoc::new('A', 15));
    }

    #[test]
    fn compact_numeral_tokens_expand_on_their_letters() {
        assert_eq!(gen_title("100best", 3).unwrap(), CutterLoc::new('A', 153));
        assert_eq!(gen_title("1880's", 3).unwrap(), CutterLoc::new('A', 157));
    }

    #[test]
    fn numeral_with_unnormalizable_remainder_is_bare_a15() {
        assert_eq!(gen_title("42 !!!", 3).unwrap(), CutterLoc::new('A', 15));
    }
}
