//! The matcher behind [`Language::strip_initial_article`]: where a title's
//! initial article starts, where it ends, and where the filing title begins.

use super::Language;

/// How an article meets the word after it, from its last character.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArticleKind {
    /// A separate word, which whitespace must follow (`the`, `der`, `'t`).
    Word,
    /// It ends in an apostrophe, and whitespace may follow (`l'`, `un'`).
    Elided,
    /// It ends in a hyphen, and the noun follows directly (Maltese `il-`).
    Joined,
}

impl ArticleKind {
    /// The kind of `article`, a lowercase entry of
    /// [`Language::articles`].
    fn of(article: &str) -> Self {
        match article.chars().next_back() {
            Some('\'') => Self::Elided,
            Some('-') => Self::Joined,
            _ => Self::Word,
        }
    }
}

/// Whether some method files on `c`: Cutter-Sanborn files on letters, and the
/// `LoC` numeral rule takes ASCII digits.
fn is_filing(c: char) -> bool {
    c.is_alphabetic() || c.is_ascii_digit()
}

/// The characters an apostrophe in an article matches: the ASCII apostrophe,
/// the right and left single quotation marks (word processors turn a
/// word-initial apostrophe into the left one), the modifier letter
/// apostrophe, and the grave and acute accents typed for it.
fn is_apostrophe(c: char) -> bool {
    matches!(
        c,
        '\'' | '\u{2019}' | '\u{2018}' | '\u{2BC}' | '\u{60}' | '\u{B4}'
    )
}

/// The characters a hyphen in an article matches: the hyphen-minus, the
/// hyphen and the non-breaking hyphen.
fn is_hyphen(c: char) -> bool {
    matches!(c, '-' | '\u{2010}' | '\u{2011}')
}

/// Whether the title character `c` matches the article character `a`
/// (lowercase, precomposed): apostrophes and hyphens by class, and anything
/// else when `c` lowercases to exactly one char, equal to `a`. There is no
/// diacritic folding, so Portuguese `À` doesn't match `a`.
fn char_matches(c: char, a: char) -> bool {
    match a {
        '\'' => is_apostrophe(c),
        '-' => is_hyphen(c),
        _ => {
            let mut lower = c.to_lowercase();
            lower.next() == Some(a) && lower.next().is_none()
        }
    }
}

/// The rest of `s` after `article`, when `s` starts with it and the
/// article's boundary holds: whitespace after a word article, anything after
/// an elided one, and a non-whitespace character after a joined one.
fn match_article<'t>(s: &'t str, article: &str) -> Option<&'t str> {
    let mut chars = s.chars();
    for a in article.chars() {
        if !char_matches(chars.next()?, a) {
            return None;
        }
    }
    let rest = chars.as_str();
    let next = rest.chars().next();
    let boundary = match ArticleKind::of(article) {
        ArticleKind::Word => next.is_some_and(char::is_whitespace),
        ArticleKind::Elided => true,
        ArticleKind::Joined => next.is_some_and(|c| !c.is_whitespace()),
    };
    boundary.then_some(rest)
}

/// `s` from its first filing character, or `None` when it has none.
fn from_first_filing(s: &str) -> Option<&str> {
    let rest = s.trim_start_matches(|c: char| !is_filing(c));
    (!rest.is_empty()).then_some(rest)
}

/// `rest` without the Irish mutation prefix that may follow `article`: `t`
/// after `an`, `h` after `na`. `rest` starts at a filing character.
///
/// A prefix followed by a hyphen goes in any case (`an t-uisce`). A bare one
/// goes only when it is lowercase and the next character shows it is a
/// prefix: an uppercase letter for both (`An tUasal`, `na hÉireann`), `s`
/// after `t` (`an tsráid`), and a lowercase vowel after `h` (`na hoíche`).
/// Nothing changes when no filing character would remain.
fn strip_irish_prefix<'t>(article: &str, rest: &'t str) -> &'t str {
    let (prefix, marks_bare_prefix): (char, fn(char) -> bool) = match article {
        "an" => ('t', |c| c.is_uppercase() || c == 's'),
        "na" => ('h', |c| {
            c.is_uppercase()
                || matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'á' | 'é' | 'í' | 'ó' | 'ú')
        }),
        _ => return rest,
    };
    let mut chars = rest.chars();
    let Some(first) = chars.next() else {
        return rest;
    };
    if first.to_ascii_lowercase() != prefix {
        return rest;
    }
    let after = chars.as_str();
    let mut following = after.chars();
    match following.next() {
        Some(c) if is_hyphen(c) => from_first_filing(following.as_str()).unwrap_or(rest),
        Some(c) if first == prefix && marks_bare_prefix(c) => after,
        Some(_) | None => rest,
    }
}

/// `title` without its initial article in `language`, as
/// [`Language::strip_initial_article`] documents.
pub(super) fn strip(language: Language, title: &str) -> Option<&str> {
    let articles = language.articles();
    // Every position up to and including the first filing character:
    // whatever precedes it is passed over, and an apostrophe-initial
    // article can match inside quotes. Past the first filing character,
    // no article can be initial.
    let mut candidate = title;
    let mut chars = title.chars();
    while let Some(c) = chars.next() {
        for &article in articles {
            if let Some(rest) = match_article(candidate, article) {
                let filing = from_first_filing(rest)?;
                return Some(if language == Language::Irish {
                    strip_irish_prefix(article, filing)
                } else {
                    filing
                });
            }
        }
        if is_filing(c) {
            break;
        }
        candidate = chars.as_str();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    use Language as L;

    /// Uppercases the first alphabetic character (`'t` -> `'T`).
    fn title_case(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        let mut done = false;
        for c in s.chars() {
            if !done && c.is_alphabetic() {
                out.extend(c.to_uppercase());
                done = true;
            } else {
                out.push(c);
            }
        }
        out
    }

    /// `article` in lowercase, Title Case and UPPERCASE.
    fn cases(article: &str) -> [String; 3] {
        [
            article.to_owned(),
            title_case(article),
            article.to_uppercase(),
        ]
    }

    /// Asserts that `language` strips `input` to `want`, a subslice of it.
    #[track_caller]
    fn strips(language: Language, input: &str, want: &str) {
        let got = language.strip_initial_article(input);
        assert_eq!(got, Some(want), "{language} {input:?}");
        if let Some(out) = got {
            let range = input.as_bytes().as_ptr_range();
            let out_range = out.as_bytes().as_ptr_range();
            assert!(
                range.start <= out_range.start && out_range.end <= range.end,
                "{out:?} is not a subslice of {input:?}"
            );
        }
    }

    /// Asserts that `language` strips nothing from `input`.
    #[track_caller]
    fn keeps(language: Language, input: &str) {
        assert_eq!(
            language.strip_initial_article(input),
            None,
            "{language} {input:?}"
        );
    }

    #[test]
    fn every_article_strips_per_kind() {
        for &language in L::ALL {
            for &article in language.articles() {
                for art in cases(article) {
                    match ArticleKind::of(article) {
                        ArticleKind::Word => {
                            strips(language, &format!("{art} word"), "word");
                            keeps(language, &format!("{art}word"));
                        }
                        ArticleKind::Elided => {
                            strips(language, &format!("{art}word"), "word");
                            strips(language, &format!("{art} word"), "word");
                        }
                        ArticleKind::Joined => {
                            strips(language, &format!("{art}word"), "word");
                            keeps(language, &format!("{art} word"));
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn excluded_forms_do_not_strip() {
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
            (L::Portuguese, &["uns", "umas", "à", "ao"]),
            (L::Galician, &["uns", "unhas"]),
            (L::Luxembourgish, &["dem", "der", "engem", "enger"]),
        ];
        for &(language, forms) in excluded {
            for form in forms {
                for art in cases(form) {
                    keeps(language, &format!("{art} word"));
                    keeps(language, &format!("{art}word"));
                }
            }
        }
    }

    #[test]
    fn boundaries() {
        keeps(L::English, "Theory");
        keeps(L::English, "Anthem");
        keeps(L::Spanish, "Elegía");
        keeps(L::English, "A, B, C");
        // `…` (U+2026) is not whitespace, so the word boundary fails.
        keeps(L::Spanish, "¿El…?");
        for apostrophe in ['\'', '\u{2019}', '\u{2018}', '\u{2BC}', '\u{60}', '\u{B4}'] {
            strips(L::French, &format!("L{apostrophe}amour"), "amour");
            strips(L::French, &format!("L{apostrophe} amour"), "amour");
        }
        keeps(L::French, "L\"amour");
        for hyphen in ['-', '\u{2010}', '\u{2011}'] {
            strips(L::Maltese, &format!("il{hyphen}Kantilena"), "Kantilena");
        }
        keeps(L::Maltese, "il\u{2013}Kantilena"); // an en dash is not a hyphen
        keeps(L::Maltese, "il- Kantilena");
        strips(L::Maltese, "IL-KANTILENA", "KANTILENA");
        strips(L::Maltese, "il-\"Kantilena\"", "Kantilena\"");
        strips(L::Maltese, "il-1", "1");
        strips(L::Maltese, "ix-Xemx", "Xemx");
        strips(L::Maltese, "l-Ilma", "Ilma");
        keeps(L::Maltese, "il-");
        for space in [' ', '\t', '\u{A0}', '\u{202F}', '\u{3000}'] {
            strips(L::English, &format!("The{space}Bear"), "Bear");
        }
    }

    #[test]
    fn leading_and_following_punctuation() {
        strips(
            L::French,
            "«\u{202F}Le Petit Prince\u{202F}»",
            "Petit Prince\u{202F}»",
        );
        strips(L::Spanish, "¿El otoño?", "otoño?");
        strips(L::English, "\"The Bear\"", "Bear\"");
        strips(
            L::English,
            "[The part of Pennsylvania]",
            "part of Pennsylvania]",
        );
        strips(L::English, "  The Bear", "Bear");
        strips(L::English, "* The Bear", "Bear");
        strips(L::English, "½ The Bear", "Bear");
        // An apostrophe-initial article inside quotes strips, quote and all.
        strips(L::Dutch, "'t Hooft", "Hooft");
        strips(L::Dutch, "‘t Hooft", "Hooft");
        strips(L::Dutch, "\"'t Hooft\"", "Hooft\"");
        strips(L::Afrikaans, "«'n Mens»", "Mens»");
        strips(L::Dutch, "‘De Avonden’", "Avonden’");
        // Punctuation after the article is nonfiling too.
        strips(L::English, "The \"Bear\"", "Bear\"");
        strips(L::English, "The #1", "1");
        strips(L::English, "The (2)", "2)");
        strips(L::English, "The ½ Bear", "Bear");
        keeps(L::English, "The ½");
        // An article is only initial: none is found after the first word.
        keeps(L::English, "Bear the cat");
        keeps(L::English, "Theory of the bear");
        keeps(L::French, "Bonjour la vie");
        // Without an article, leading punctuation stays.
        keeps(L::English, "\"Bear\"");
        keeps(L::English, "... as others see us");
    }

    #[test]
    fn edge_cases() {
        for title in ["The", "The ", "The !", "", "   "] {
            keeps(L::English, title);
        }
        keeps(L::French, "Les");
        keeps(L::French, "L'");
        keeps(L::French, "L' ");
        // Only one article is removed.
        strips(L::English, "The A Team", "A Team");
        strips(L::English, "The The", "The");
        // Documented false positive: a letter used as a word.
        strips(L::English, "A to Z drug facts", "to Z drug facts");
        // An article in NFD never matches; NFD elsewhere doesn't matter.
        keeps(L::Spanish, "E\u{301}l libro");
        keeps(L::Spanish, "La\u{301} casa");
        strips(L::English, "The Cafe\u{301}", "Cafe\u{301}");
        // A multi-char lowercase form (İ -> i + U+0307) never matches.
        keeps(L::Italian, "İl libro");
        // Diacritics are matched, not folded.
        strips(L::Luxembourgish, "Déi Geschicht", "Geschicht");
        strips(L::Icelandic, "HIÐ ÍSLENZKA", "ÍSLENZKA");
        strips(L::Maltese, "IŻ-ŻMIEN", "ŻMIEN");
        keeps(L::Luxembourgish, "Dei Geschicht");
        keeps(L::Portuguese, "À espera");
        // The documented limits: a non-ASCII digit doesn't end the leading
        // run, and an invisible format character hides the article.
        strips(L::English, "\u{FF11} The Bear", "Bear");
        keeps(L::English, "1 The Bear");
        keeps(L::English, "The\u{200B} Bear");
        strips(L::English, "\u{200B}The Bear", "Bear");
        // Long inputs are fine.
        let long = format!("The {}", "a".repeat(100_000));
        assert_eq!(
            L::English.strip_initial_article(&long).map(str::len),
            Some(100_000)
        );
        let punctuation = "!".repeat(100_000);
        keeps(L::English, &punctuation);
    }

    #[test]
    fn irish() {
        strips(L::Irish, "an t-uisce", "uisce");
        strips(L::Irish, "An T-Uisce", "Uisce");
        strips(L::Irish, "AN T-UISCE", "UISCE");
        strips(L::Irish, "an t\u{2011}uisce", "uisce");
        strips(L::Irish, "An tUasal", "Uasal");
        strips(L::Irish, "an tsráid", "sráid");
        strips(L::Irish, "an tSeapáin", "Seapáin");
        strips(L::Irish, "an \"tUasal\"", "Uasal\"");
        strips(L::Irish, "na hoíche", "oíche");
        strips(L::Irish, "na h-oíche", "oíche");
        strips(L::Irish, "na hÉireann", "Éireann");
        strips(L::Irish, "na hÉ", "É");
        // Every vowel after a bare `h`.
        strips(L::Irish, "na háite", "áite");
        strips(L::Irish, "na heaglaise", "eaglaise");
        strips(L::Irish, "na hiníne", "iníne");
        strips(L::Irish, "na hóige", "óige");
        strips(L::Irish, "na huaire", "uaire");
        strips(L::Irish, "na hé", "é");
        strips(L::Irish, "na hí", "í");
        strips(L::Irish, "na húlla", "úlla");
        // A native `t`/`h` is kept, and so is a bare prefix in capitals.
        strips(L::Irish, "an teach", "teach");
        strips(L::Irish, "an tuairim", "tuairim");
        strips(L::Irish, "na leabhair", "leabhair");
        strips(L::Irish, "AN TÁIN", "TÁIN");
        strips(L::Irish, "NA HOÍCHE", "HOÍCHE");
        // Documented false positives: loanwords that begin with t or h.
        strips(L::Irish, "An tsunami", "sunami");
        strips(L::Irish, "na hataí", "ataí");
        // A prefix with nothing filing after it stays.
        strips(L::Irish, "an t-", "t-");
        strips(L::Irish, "an t-!", "t-!");
        strips(L::Irish, "na h", "h");
        // The prefix rule is Irish only: the other languages that list `an`
        // keep the letter after it.
        let with_an: Vec<Language> = L::ALL
            .iter()
            .copied()
            .filter(|&language| language != L::Irish && language.articles().contains(&"an"))
            .collect();
        assert!(!with_an.is_empty());
        for language in with_an {
            strips(language, "An tUasal", "tUasal");
        }
    }

    #[test]
    fn languages_without_articles_strip_nothing() {
        for language in [L::Latin, L::Albanian] {
            for title in [
                "Le Bear",
                "The Bear",
                "De bello Gallico",
                "'t Hooft",
                "Një natë",
                "il-Kantilena",
            ] {
                keeps(language, title);
            }
        }
    }

    #[test]
    fn one_match_per_position() {
        // When one article is a prefix of another, the shorter is a word
        // article and the longer continues with a non-whitespace character
        // (`un`/`un'`/`una`), so the shorter's boundary fails wherever the
        // longer matches: at most one article matches at any position.
        for &language in L::ALL {
            for &short in language.articles() {
                for &long in language.articles() {
                    if let Some(tail) = long.strip_prefix(short).filter(|t| !t.is_empty()) {
                        assert_eq!(
                            ArticleKind::of(short),
                            ArticleKind::Word,
                            "{language} {short}/{long}"
                        );
                        assert!(
                            !tail.starts_with(char::is_whitespace),
                            "{language} {short}/{long}"
                        );
                    }
                }
            }
        }
    }
}
