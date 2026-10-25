//! The language of a work, for csan's language-dependent filing rules.
//!
//! Filing rules that depend on the language of the title are opt-in and
//! keyed by a [`Language`], which lists the supported languages and their
//! codes and parses an ISO 639 code, a BCP 47 tag or a POSIX locale.
//! Frontends that take a language and an on/off switch per rule convert them
//! with [`Language::from_optional`] and [`Language::for_rule`].
//!
//! # Initial articles
//!
//! Library filing practice ignores a title's initial article. Cutter's
//! _Rules for a Dictionary Catalogue_ say to "take account of every word
//! except initial articles", and LC's shelflisting manual (G 100 §9) says to
//! "ignore initial articles in the nominative case". So _A Christmas Carol_
//! files under _Christmas_. [`Language::strip_initial_article`] removes the
//! article, and the table in [`Language`]'s documentation lists each
//! language's articles. A title is filed without it through
//! [`Opus::with_article_language`](crate::Opus::with_article_language) (or the
//! same method on [`Attribution`](crate::Attribution)). Author names are
//! never stripped.
//!
//! A word that is an article in one language is an ordinary word in another
//! (English _Die Hard_), which is why the rule is keyed by the title's
//! language.
//!
//! The lists start from MARC 21 Appendix F's forms, nominative only (G 100
//! §9), and depart from it where the sources do:
//! - added: Breton (the list the JSC approved for RDA in 2006),
//!   Luxembourgish (its grammar's nominative forms, as the official
//!   orthography spells them), and standard forms that fill a gap in a
//!   paradigm Appendix F otherwise gives (Galician _os_ beside _as_, _un_
//!   beside _unha_; Provençal _lei_, _leis_; the assimilated Maltese forms;
//!   Irish without the hyphen);
//! - left out: forms G 100 or the grammars show aren't initial articles
//!   (Catalan's personal _en_, Romanian's genitival _a_, _al_, and
//!   Albanian's _njē_, as G 100 §9 lists Albanian as having no initial
//!   articles), forms with no grammatical basis (Icelandic _'r_, Provençal
//!   _ih'_), and archaic or dialect forms that are mostly other words
//!   (English _ye_, _d'_, _de_; German _'s_, usually the pronoun _es_).
//!
//! Appendix F gives no plural indefinite for Spanish, Portuguese, Catalan,
//! Galician or French (no _des_), only for Occitan (_uns_, _us_). That
//! omission is taken as deliberate, so _unos_, _unas_, _uns_, _umas_, _unes_
//! and _unhas_ aren't added, while Occitan's stay. Numerals that are also
//! articles stay in the lists, so they are stripped, because G 100 makes "no
//! attempt … to differentiate cardinal numbers from articles". Albanian and
//! Latin, which G 100 §9 lists as having no initial articles, have none.
//!
//! Known false positives, which are the reason stripping is opt-in:
//! - a letter used as a word (_A to Z drug facts_);
//! - pronouns (_Lo dijo_, _O iubesc_, _Das ist…_);
//! - articles that are part of a name (_El Cid_, _Los Angeles_), which
//!   G 100 §10 keeps;
//! - the Irish interrogative _An_, and Irish loanwords that begin with `t`
//!   or `h` (_An tsunami_, _na hataí_).
//!
//! Known false negatives:
//! - all-caps Irish mutation prefixes;
//! - the oblique-case articles (German, Icelandic, Luxembourgish, Romanian),
//!   deliberately, per G 100's nominative rule (_Dem Andenken…_ files under
//!   _Dem_), and the plural indefinites above;
//! - articles written in other Unicode forms: decomposed (NFD) or
//!   compatibility forms such as fullwidth letters are not folded;
//! - an invisible format character (U+200B, U+00AD, U+FEFF) between an
//!   article and its space.

mod articles;

use std::fmt::{self, Display, Formatter, Write as _};
use std::str::FromStr;

/// Generates [`Language`] from one row per language, so the enum,
/// [`Language::ALL`], the lookups and the table in the enum's
/// documentation can't disagree.
///
/// A row holds optional notes for the variant's documentation, the variant,
/// its English name, its codes (canonical first) and its articles. The
/// variant's documentation starts with its name and canonical code, so those
/// can't drift either. Rows go in code order, which is the order of `ALL`.
/// rustfmt doesn't format these rows; they are aligned by hand.
macro_rules! languages {
    ($(
        $(#[doc = $note:literal])*
        $variant:ident, $name:literal,
        [$code0:literal $(, $code:literal)*],
        [$($article0:literal $(, $article:literal)*)?];
    )+) => {
        /// The language of a work, for csan's language-dependent filing rules.
        ///
        /// Parse one from an ISO 639 code, a BCP 47 tag or a POSIX locale with
        /// [`FromStr`] (`"pt-BR".parse()`), or from an optional user input with
        /// [`Language::from_optional`]. It displays as its canonical code.
        /// [`Language::strip_initial_article`] removes a title's initial
        /// article in that language; the table gives each language's codes and
        /// articles.
        ///
        /// | Language | Codes | Articles |
        /// | -------- | ----- | -------- |
        $(
            #[doc = concat!(
                "| ", $name,
                " | `", $code0, "`", $(", `", $code, "`",)*
                " | ", $("`", $article0, "`", $(", `", $article, "`",)*)? " |"
            )]
        )+
        ///
        /// An empty Articles cell means the language has no initial articles:
        /// its codes are accepted, and nothing is stripped.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[non_exhaustive]
        pub enum Language {
            $(
                #[doc = concat!($name, ", code `", $code0, "`.")]
                $(#[doc = $note])*
                $variant,
            )+
        }

        impl Language {
            /// Every supported language, ordered by code.
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];

            /// The English name, e.g. `"Afrikaans"`, `"Catalan (Valencian)"`.
            #[must_use]
            pub const fn name(self) -> &'static str {
                match self {
                    $(Self::$variant => $name,)+
                }
            }

            /// The canonical code: the first of [`codes`](Self::codes), the
            /// ISO 639-1 code (e.g. `"en"`). It is also what [`Display`]
            /// prints.
            #[must_use]
            pub const fn code(self) -> &'static str {
                match self {
                    $(Self::$variant => $code0,)+
                }
            }

            /// Every accepted code, lowercase, canonical first, e.g.
            /// `["nl", "nld", "dut"]`.
            #[must_use]
            pub const fn codes(self) -> &'static [&'static str] {
                match self {
                    $(Self::$variant => &[$code0 $(, $code)*],)+
                }
            }

            /// The articles recognized at the start of a title: lowercase,
            /// precomposed (NFC), as written. Elided forms end in an
            /// apostrophe (`l'`) and joined forms in a hyphen (Maltese `il-`).
            /// Irish returns `["an", "na"]`; the `t`/`h` prefixes that follow
            /// them are a rule, not list entries. Albanian and Latin return
            /// `[]`.
            #[must_use]
            pub const fn articles(self) -> &'static [&'static str] {
                match self {
                    $(Self::$variant => &[$($article0 $(, $article)*)?],)+
                }
            }
        }
    };
}

languages! {
    Afrikaans, "Afrikaans", ["af", "afr"], ["die", "'n"];
    Breton, "Breton", ["br", "bre"], ["an", "al", "ar", "un", "ul", "ur", "eun", "eul", "eur"];
    Catalan, "Catalan (Valencian)", ["ca", "cat"], ["el", "els", "la", "les", "l'", "un", "una"];
    Welsh, "Welsh", ["cy", "cym", "wel"], ["y", "yr"];
    Danish, "Danish", ["da", "dan"], ["de", "den", "det", "en", "et"];
    German, "German", ["de", "deu", "ger"], ["der", "die", "das", "ein", "eine"];
    English, "English", ["en", "eng"], ["a", "an", "the"];
    Esperanto, "Esperanto", ["eo", "epo"], ["la"];
    Spanish, "Spanish", ["es", "spa"], ["el", "la", "los", "las", "lo", "un", "una"];
    French, "French", ["fr", "fra", "fre"], ["le", "la", "les", "l'", "un", "une"];
    /// Its articles may be followed by a `t` or `h` mutation prefix, which is
    /// disregarded with them.
    Irish, "Irish", ["ga", "gle"], ["an", "na"];
    Galician, "Galician", ["gl", "glg"], ["o", "a", "os", "as", "un", "unha"];
    Hungarian, "Hungarian", ["hu", "hun"], ["a", "az", "egy"];
    Icelandic, "Icelandic", ["is", "isl", "ice"], ["hinn", "hin", "hið", "hinir", "hinar"];
    Italian, "Italian", ["it", "ita"],
        ["il", "lo", "la", "l'", "i", "gli", "gl'", "le", "un", "uno", "una", "un'"];
    /// It has no articles (LC G 100 §9): its codes are accepted, and nothing
    /// is stripped.
    Latin, "Latin", ["la", "lat"], [];
    Luxembourgish, "Luxembourgish", ["lb", "ltz"],
        ["den", "de", "d'", "deen", "dat", "déi", "en", "e", "eng"];
    /// Its article is joined to the noun by a hyphen.
    Maltese, "Maltese", ["mt", "mlt"],
        ["il-", "l-", "iċ-", "id-", "in-", "ir-", "is-", "it-", "ix-", "iż-", "iz-", "ic-"];
    Dutch, "Dutch (Flemish)", ["nl", "nld", "dut"], ["de", "het", "een", "eene", "'t", "'n"];
    /// Bokmål and Nynorsk, in one list.
    Norwegian, "Norwegian", ["no", "nor", "nb", "nob", "nn", "nno"],
        ["de", "dei", "den", "det", "e", "ei", "ein", "eit", "en", "et"];
    Occitan, "Occitan (Provençal)", ["oc", "oci", "pro"],
        ["il", "l'", "la", "las", "le", "les", "lh", "lhi", "li", "lis", "lo", "los", "lou",
         "lu", "lei", "leis", "un", "una", "uno", "uns", "us"];
    Portuguese, "Portuguese", ["pt", "por"], ["o", "a", "os", "as", "um", "uma"];
    Romanian, "Romanian (Moldavian)", ["ro", "ron", "rum", "mo", "mol"], ["un", "o"];
    /// It has no initial articles (LC G 100 §9): its codes are accepted, and
    /// nothing is stripped.
    Albanian, "Albanian", ["sq", "sqi", "alb", "als", "aln", "aae", "aat"], [];
    Swedish, "Swedish", ["sv", "swe"], ["de", "den", "det", "en", "ett"];
}

impl Language {
    /// `title` without its initial article, when it starts with one that this
    /// language recognizes and a letter or ASCII digit follows; otherwise
    /// `None`, including for a title that is only an article. The result is a
    /// subslice of `title`, so nothing is allocated.
    ///
    /// The article may follow anything but a letter or ASCII digit: spaces,
    /// punctuation, symbols (`"¿El otoño?"`, `"'t Hooft"` in quotes). A word
    /// article must then be followed by whitespace (`"The, Bear"` keeps its
    /// _The_), an elided one by anything (`"L'amour"`, `"L' amour"`), and a
    /// joined one by a non-space (`"il-Kantilena"`). The result starts at the
    /// first letter or ASCII digit after it, since MARC counts the spaces and
    /// punctuation around an initial article as nonfiling too. Letters match
    /// regardless of case but with their diacritics, an apostrophe matches its
    /// typographic variants, and a hyphen its Unicode forms. Irish also loses
    /// the `t`/`h` mutation prefix after its article. Only one article is
    /// removed.
    ///
    /// Three limits follow from these rules. A digit other than an ASCII one
    /// doesn't end the leading run, so `"１ The Bear"` (a fullwidth one)
    /// loses its _The_. An invisible format character between an article and
    /// its space (U+200B, U+00AD, U+FEFF) hides the article, so nothing is
    /// stripped. And an article in another Unicode form (decomposed, or
    /// fullwidth `"Ｔｈｅ"`) doesn't match: there is no normalization.
    ///
    /// # Examples
    /// ```
    /// use csan_core::Language;
    ///
    /// let english = Language::English;
    /// assert_eq!(english.strip_initial_article("The Bear"), Some("Bear"));
    /// assert_eq!(english.strip_initial_article("\"The 100 best\""), Some("100 best\""));
    /// assert_eq!(english.strip_initial_article("Theory"), None);
    /// assert_eq!(english.strip_initial_article("The"), None);
    ///
    /// let french = Language::French;
    /// assert_eq!(french.strip_initial_article("L’Étranger"), Some("Étranger"));
    /// let irish = Language::Irish;
    /// assert_eq!(irish.strip_initial_article("An tUasal"), Some("Uasal"));
    /// ```
    #[must_use]
    pub fn strip_initial_article(self, title: &str) -> Option<&str> {
        articles::strip(self, title)
    }

    /// The front door for frontends, like
    /// [`Attribution::from_parts`](crate::Attribution::from_parts): `None` and
    /// blank strings mean no language (`Ok(None)`), and anything else parses
    /// with [`FromStr`].
    ///
    /// # Errors
    /// Returns [`UnknownLanguage`] for a non-blank code that no
    /// supported language uses.
    ///
    /// # Examples
    /// ```
    /// use csan_core::Language;
    ///
    /// assert_eq!(Language::from_optional(None), Ok(None));
    /// assert_eq!(Language::from_optional(Some("  ")), Ok(None));
    /// assert_eq!(
    ///     Language::from_optional(Some("pt-BR")),
    ///     Ok(Some(Language::Portuguese))
    /// );
    /// assert!(Language::from_optional(Some("xx")).is_err());
    /// ```
    pub fn from_optional(code: Option<&str>) -> Result<Option<Self>, UnknownLanguage> {
        match code.map(str::trim) {
            None | Some("") => Ok(None),
            Some(code) => code.parse().map(Some),
        }
    }

    /// The language that a rule applies with, for frontends that take a
    /// language and an on/off switch per rule (the CLI's `--lang` and
    /// `--strip-article`): `Ok(None)` when the switch is off, and the language
    /// when it is on.
    ///
    /// A language with the switch off is not an error, so that a frontend can
    /// set the language once (Typst's `cutter-sanborn.with(lang: "pt")`, a
    /// shell alias) and turn rules on per call.
    ///
    /// # Errors
    /// Returns [`MissingLanguage`] when the switch is on and no language is
    /// given.
    ///
    /// # Examples
    /// ```
    /// use csan_core::{Language, MissingLanguage};
    ///
    /// let english = Some(Language::English);
    /// assert_eq!(Language::for_rule(english, true), Ok(english));
    /// assert_eq!(Language::for_rule(english, false), Ok(None));
    /// assert_eq!(Language::for_rule(None, false), Ok(None));
    /// assert_eq!(Language::for_rule(None, true), Err(MissingLanguage));
    /// ```
    pub const fn for_rule(
        language: Option<Self>,
        enabled: bool,
    ) -> Result<Option<Self>, MissingLanguage> {
        match (enabled, language) {
            (false, _) => Ok(None),
            (true, Some(_)) => Ok(language),
            (true, None) => Err(MissingLanguage),
        }
    }
}

impl FromStr for Language {
    type Err = UnknownLanguage;

    /// Parses an ISO 639 code, a BCP 47 tag or a POSIX locale, in any case.
    ///
    /// The input is trimmed and cut at its first `-`, `_`, `.` or `@`, and
    /// that primary subtag is looked up among every language's
    /// [`codes`](Self::codes): `"pt-BR"`, `"ca-ES-valencia"`, `"ca@valencia"`
    /// and `"pt_BR.UTF-8"` all resolve. BCP 47 tags and POSIX locales are ASCII,
    /// so non-ASCII input is an error anywhere after trimming (`"en-é"`).
    ///
    /// # Examples
    /// ```
    /// use csan_core::Language;
    ///
    /// assert_eq!("EN".parse(), Ok(Language::English));
    /// assert_eq!("dut".parse(), Ok(Language::Dutch));
    /// assert_eq!("ca-valencia".parse(), Ok(Language::Catalan));
    /// assert!("val".parse::<Language>().is_err()); // ISO 639-3: Vehes
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        let unknown = || UnknownLanguage(trimmed.to_owned());
        // BCP 47 tags and POSIX locales are ASCII, so any non-ASCII input is an
        // error, even after the primary subtag ("en-é").
        if !trimmed.is_ascii() {
            return Err(unknown());
        }
        // `split` always yields at least one (possibly empty) piece.
        let primary = trimmed
            .split(['-', '_', '.', '@'])
            .next()
            .unwrap_or_default();
        Self::ALL
            .iter()
            .copied()
            .find(|language| {
                language
                    .codes()
                    .iter()
                    .any(|code| code.eq_ignore_ascii_case(primary))
            })
            .ok_or_else(unknown)
    }
}

impl Display for Language {
    /// Writes the canonical code (e.g. `en`), honouring width and fill.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.pad(self.code())
    }
}

/// The error for a code that no supported language uses.
///
/// It holds the trimmed input, in its original case. Its message lists the
/// canonical codes from [`Language::ALL`]:
/// `unknown language "xx" (expected one of: af, br, ca, …)`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, derive_more::Error)]
pub struct UnknownLanguage(#[error(ignore)] String);

impl Display for UnknownLanguage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // Escaped char by char, so that combining marks and other grapheme
        // extenders are escaped wherever they are (`str::escape_debug` leaves
        // them unescaped after the first char), without `{:?}` (`use_debug`).
        // An apostrophe is written as is, as `{:?}` on a string does (`"l'"`).
        f.write_str("unknown language \"")?;
        for c in self.0.chars() {
            if c == '\'' {
                f.write_char(c)?;
            } else {
                for escaped in c.escape_debug() {
                    f.write_char(escaped)?;
                }
            }
        }
        f.write_str("\" (expected one of: ")?;
        for (i, language) in Language::ALL.iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            f.write_str(language.code())?;
        }
        f.write_str(")")
    }
}

/// The error for a rule switched on without a language, from
/// [`Language::for_rule`].
///
/// Its message is generic (`a language is required`): a frontend reports it
/// in its own words, naming its own parameters (the CLI says `--strip-article
/// requires --lang`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, derive_more::Display, derive_more::Error)]
#[display("a language is required")]
pub struct MissingLanguage;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::error::Error;

    use Language as L;

    #[test]
    fn article_lists_are_pinned() {
        // Each list is part of the stability contract (a change in what is
        // stripped changes callers' numbers), so a change to one must also
        // change this test.
        let pinned: &[(Language, &str)] = &[
            (L::Afrikaans, "die 'n"),
            (L::Albanian, ""),
            (L::Breton, "an al ar un ul ur eun eul eur"),
            (L::Catalan, "el els la les l' un una"),
            (L::Danish, "de den det en et"),
            (L::Dutch, "de het een eene 't 'n"),
            (L::English, "a an the"),
            (L::Esperanto, "la"),
            (L::French, "le la les l' un une"),
            (L::Galician, "o a os as un unha"),
            (L::German, "der die das ein eine"),
            (L::Hungarian, "a az egy"),
            (L::Icelandic, "hinn hin hið hinir hinar"),
            (L::Irish, "an na"),
            (L::Italian, "il lo la l' i gli gl' le un uno una un'"),
            (L::Latin, ""),
            (L::Luxembourgish, "den de d' deen dat déi en e eng"),
            (L::Maltese, "il- l- iċ- id- in- ir- is- it- ix- iż- iz- ic-"),
            (L::Norwegian, "de dei den det e ei ein eit en et"),
            (
                L::Occitan,
                "il l' la las le les lh lhi li lis lo los lou lu lei leis un una uno uns us",
            ),
            (L::Portuguese, "o a os as um uma"),
            (L::Romanian, "un o"),
            (L::Spanish, "el la los las lo un una"),
            (L::Swedish, "de den det en ett"),
            (L::Welsh, "y yr"),
        ];
        assert_eq!(pinned.len(), L::ALL.len());
        for &(language, list) in pinned {
            let want: Vec<&str> = list.split_whitespace().collect();
            assert_eq!(language.articles(), want.as_slice(), "{language}");
        }
    }

    #[test]
    fn names_and_codes_are_pinned() {
        // Removing a code breaks callers that pass it, so the accepted
        // codes are pinned like the articles, and so are the names that make
        // Valencian, Flemish, Moldavian and Provençal findable in help text.
        let pinned: &[(Language, &str, &str)] = &[
            (L::Afrikaans, "Afrikaans", "af afr"),
            (L::Albanian, "Albanian", "sq sqi alb als aln aae aat"),
            (L::Breton, "Breton", "br bre"),
            (L::Catalan, "Catalan (Valencian)", "ca cat"),
            (L::Danish, "Danish", "da dan"),
            (L::Dutch, "Dutch (Flemish)", "nl nld dut"),
            (L::English, "English", "en eng"),
            (L::Esperanto, "Esperanto", "eo epo"),
            (L::French, "French", "fr fra fre"),
            (L::Galician, "Galician", "gl glg"),
            (L::German, "German", "de deu ger"),
            (L::Hungarian, "Hungarian", "hu hun"),
            (L::Icelandic, "Icelandic", "is isl ice"),
            (L::Irish, "Irish", "ga gle"),
            (L::Italian, "Italian", "it ita"),
            (L::Latin, "Latin", "la lat"),
            (L::Luxembourgish, "Luxembourgish", "lb ltz"),
            (L::Maltese, "Maltese", "mt mlt"),
            (L::Norwegian, "Norwegian", "no nor nb nob nn nno"),
            (L::Occitan, "Occitan (Provençal)", "oc oci pro"),
            (L::Portuguese, "Portuguese", "pt por"),
            (L::Romanian, "Romanian (Moldavian)", "ro ron rum mo mol"),
            (L::Spanish, "Spanish", "es spa"),
            (L::Swedish, "Swedish", "sv swe"),
            (L::Welsh, "Welsh", "cy cym wel"),
        ];
        assert_eq!(pinned.len(), L::ALL.len());
        for &(language, name, list) in pinned {
            assert_eq!(language.name(), name, "{language}");
            let want: Vec<&str> = list.split_whitespace().collect();
            assert_eq!(language.codes(), want.as_slice(), "{language}");
        }
    }

    #[test]
    fn excluded_forms_are_not_listed() {
        // The forms left out on purpose, for the reasons the module doc
        // gives; none may reappear unnoticed.
        let excluded: &[(Language, &[&str])] = &[
            (L::English, &["ye", "d'", "de"]),
            (L::Catalan, &["en", "uns", "unes"]),
            (L::Romanian, &["unui", "unei", "a", "al", "ai", "ale"]),
            (
                L::German,
                &[
                    "des", "dem", "den", "einer", "eines", "einem", "einen", "'s",
                ],
            ),
            (
                L::Icelandic,
                &[
                    "hina", "hinum", "hinni", "hinu", "hins", "hinnar", "hinna", "'r",
                ],
            ),
            (L::Occitan, &["ih'"]),
            (L::Spanish, &["unos", "unas"]),
            (L::Portuguese, &["uns", "umas"]),
            (L::Galician, &["uns", "unhas"]),
            (L::Albanian, &["një", "nje", "nji", "njē"]),
            (L::Luxembourgish, &["dem", "der", "engem", "enger"]),
        ];
        for &(language, forms) in excluded {
            for form in forms {
                assert!(
                    !language.articles().contains(form),
                    "{language} lists {form}"
                );
            }
        }
    }

    #[test]
    fn table_invariants() {
        let unique: HashSet<_> = L::ALL.iter().collect();
        assert_eq!(unique.len(), L::ALL.len(), "ALL lists a language twice");
        assert!(
            L::ALL.is_sorted_by_key(|language| language.code()),
            "ALL must be ordered by code"
        );
        let mut seen = HashSet::new();
        for &language in L::ALL {
            assert_eq!(language.codes().first(), Some(&language.code()));
            assert_eq!(language.code().len(), 2, "{language}: not ISO 639-1");
            assert_eq!(language.to_string(), language.code());
            assert_eq!(format!("{language:>4}"), format!("{:>4}", language.code()));
            for &code in language.codes() {
                assert!(seen.insert(code), "{code} belongs to two languages");
                assert!(
                    code.bytes().all(|b| b.is_ascii_lowercase()),
                    "{language}: {code:?} is not lowercase ASCII"
                );
            }
            for &article in language.articles() {
                assert_eq!(article, article.to_lowercase(), "{language} {article}");
                assert!(!article.contains(char::is_whitespace));
                // Precomposed: no combining diacritical marks (U+0300–U+036F),
                // which an NFD spelling would carry.
                assert!(
                    !article
                        .chars()
                        .any(|c| ('\u{300}'..='\u{36F}').contains(&c)),
                    "{language} {article} is not precomposed"
                );
            }
            let mut articles = HashSet::new();
            for &article in language.articles() {
                assert!(articles.insert(article), "{language} lists {article} twice");
            }
        }
    }

    #[test]
    fn every_code_resolves_in_any_case() {
        for &language in L::ALL {
            for &code in language.codes() {
                assert_eq!(code.parse(), Ok(language), "{code}");
                assert_eq!(code.to_uppercase().parse(), Ok(language), "{code}");
                let mut mixed = code.to_owned();
                mixed.get_mut(..1).map(str::make_ascii_uppercase);
                assert_eq!(mixed.parse(), Ok(language), "{mixed}");
                assert_eq!(format!(" {code}\t").parse(), Ok(language), "{code}");
            }
        }
    }

    #[test]
    fn tags_and_locales_resolve() {
        let cases = [
            ("pt-BR", L::Portuguese),
            ("ca-ES-valencia", L::Catalan),
            ("ca-valencia", L::Catalan),
            ("ca@valencia", L::Catalan),
            ("nb_NO", L::Norwegian),
            ("nn-NO", L::Norwegian),
            ("pt_BR.UTF-8", L::Portuguese),
            ("de.UTF-8", L::German),
            ("sv-s", L::Swedish),
            ("sv-SE", L::Swedish),
            ("mo", L::Romanian),
            // Unicode whitespace around the code is trimmed before the ASCII
            // check.
            ("\u{A0}en\u{3000}", L::English),
        ];
        for (code, want) in cases {
            assert_eq!(code.parse(), Ok(want), "{code}");
        }
    }

    #[test]
    fn bad_codes_are_errors() {
        // `val` (Vehes), `iri` (Rigwe) and `gag` (Gagauz) are other
        // languages in ISO 639-3; the rest aren't codes at all.
        for code in [
            "val",
            "iri",
            "gag",
            "xx",
            "",
            "  ",
            "-",
            "_en",
            "énglish",
            "en\u{301}",
            "e n",
            "en-é",
            "ca@valència",
        ] {
            let err = code.parse::<Language>().unwrap_err();
            assert_eq!(err, UnknownLanguage(code.trim().to_owned()));
        }
        // "eng-" is the code `eng` followed by an empty subtag, not an error.
        assert_eq!("eng-".parse(), Ok(L::English));
    }

    #[test]
    fn error_message_lists_the_codes() {
        let message = "XX".parse::<Language>().unwrap_err().to_string();
        let codes: Vec<&str> = L::ALL.iter().map(|language| language.code()).collect();
        assert_eq!(
            message,
            format!(
                "unknown language \"XX\" (expected one of: {})",
                codes.join(", ")
            )
        );
        // Control characters and quotes in the input are escaped.
        let escaped = UnknownLanguage("a\"b\n".to_owned()).to_string();
        assert!(
            escaped.starts_with(r#"unknown language "a\"b\n" ("#),
            "{escaped}"
        );
        // An apostrophe is written as is, as `{:?}` would.
        let apostrophe = UnknownLanguage("l'".to_owned()).to_string();
        assert!(
            apostrophe.starts_with(r#"unknown language "l'" ("#),
            "{apostrophe}"
        );
        // A combining mark is escaped wherever it is, not only first.
        let combining = UnknownLanguage("en\u{301}x\u{301}".to_owned()).to_string();
        assert!(
            combining.starts_with(r#"unknown language "en\u{301}x\u{301}" ("#),
            "{combining}"
        );
    }

    #[test]
    fn from_optional_treats_blank_as_absent() {
        assert_eq!(L::from_optional(None), Ok(None));
        assert_eq!(L::from_optional(Some("")), Ok(None));
        assert_eq!(L::from_optional(Some(" \t ")), Ok(None));
        assert_eq!(L::from_optional(Some(" EN ")), Ok(Some(L::English)));
        assert_eq!(
            L::from_optional(Some("xx")),
            Err(UnknownLanguage("xx".to_owned()))
        );
    }

    #[test]
    fn only_languages_without_articles_have_none() {
        // Every language carries reviewed article data. An empty list is
        // allowed only for the languages G 100 §9 lists as having no initial
        // articles, so that a language added for another rule can't silently
        // strip nothing.
        let without: Vec<Language> = L::ALL
            .iter()
            .copied()
            .filter(|language| language.articles().is_empty())
            .collect();
        assert_eq!(without, [L::Latin, L::Albanian]);
    }

    #[test]
    fn for_rule_needs_a_language_only_when_on() {
        for language in [None, Some(L::English)] {
            assert_eq!(L::for_rule(language, false), Ok(None));
        }
        for &language in L::ALL {
            assert_eq!(L::for_rule(Some(language), true), Ok(Some(language)));
        }
        assert_eq!(L::for_rule(None, true), Err(MissingLanguage));
        assert_eq!(MissingLanguage.to_string(), "a language is required");
    }

    #[test]
    fn errors_are_send_sync_and_std_error() {
        fn assert_traits<T: Error + Send + Sync + 'static>() {}
        assert_traits::<UnknownLanguage>();
        assert_traits::<MissingLanguage>();
    }
}
