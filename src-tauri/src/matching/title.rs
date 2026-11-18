//! Title normalisation for candidate finding (spec 7.2). Never removes numbers, roman numerals,
//! years or subtitles: identity is not decided here, only candidates are found.

use unicode_normalization::UnicodeNormalization;

/// NFKD without diacritics, lowercase, no ™ ® ©, `&` -> `and`, punctuation -> space.
pub fn normalize(title: &str) -> String {
    let mut out = String::with_capacity(title.len());
    // Marks go first: NFKD would turn ™ into the letters "tm".
    let cleaned: String = title
        .chars()
        .filter(|c| !matches!(c, '™' | '®' | '©' | '℠'))
        .collect();
    for c in cleaned.nfkd() {
        if unicode_normalization::char::is_combining_mark(c) {
            continue;
        }
        match c {
            '&' => out.push_str(" and "),
            c if c.is_alphanumeric() => out.extend(c.to_lowercase()),
            _ => out.push(' '),
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Words that name an edition of the same game. A larger edition may cover a smaller one.
const EDITION_WORDS: &[&str] = &[
    "gold",
    "goty",
    "complete",
    "definitive",
    "deluxe",
    "ultimate",
    "premium",
    "enhanced",
    "standard",
    "special",
    "collectors",
    "edition",
    "game of the year",
];

/// Words that name a different product (a remaster is not the original, spec 7.3).
const REMASTER_WORDS: &[&str] = &[
    "remastered",
    "remaster",
    "remake",
    "ultra",
    "hd",
    "anniversary",
];

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Stripped {
    /// Normalised title with edition and remaster words removed. Candidate-finding only.
    pub base: String,
    pub edition_words: Vec<String>,
    pub remaster_words: Vec<String>,
}

pub fn strip_editions(normalized: &str) -> Stripped {
    let mut s = format!(" {normalized} ");
    let mut editions = Vec::new();
    let mut remasters = Vec::new();
    // Multi-word phrases first.
    let mut all: Vec<(&str, bool)> = EDITION_WORDS.iter().map(|w| (*w, false)).collect();
    all.extend(REMASTER_WORDS.iter().map(|w| (*w, true)));
    all.sort_by_key(|(w, _)| std::cmp::Reverse(w.len()));
    for (w, is_remaster) in all {
        let pat = format!(" {w} ");
        while s.contains(&pat) {
            s = s.replacen(&pat, " ", 1);
            if is_remaster {
                remasters.push(w.to_owned())
            } else {
                editions.push(w.to_owned())
            }
        }
    }
    Stripped {
        base: s.split_whitespace().collect::<Vec<_>>().join(" "),
        edition_words: editions,
        remaster_words: remasters,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folds_case_accents_marks_and_ampersand() {
        assert_eq!(normalize("Pokémon™ Épico"), "pokemon epico");
        assert_eq!(normalize("Fire & Ice®"), "fire and ice");
        assert_eq!(normalize("Título: Gold-Edition!"), "titulo gold edition");
    }

    #[test]
    fn keeps_numbers_roman_numerals_and_years() {
        assert_eq!(normalize("Título II"), "titulo ii");
        assert_eq!(normalize("Título 2"), "titulo 2");
        assert_eq!(normalize("Racer 2077"), "racer 2077");
    }

    #[test]
    fn strips_edition_words_only_for_candidates() {
        let s = strip_editions(&normalize("Título: Game of the Year Edition"));
        assert_eq!(s.base, "titulo");
        assert!(s.edition_words.contains(&"game of the year".to_string()));
        let s = strip_editions(&normalize("Título Remastered"));
        assert_eq!(s.base, "titulo");
        assert_eq!(s.remaster_words, ["remastered"]);
    }

    #[test]
    fn never_strips_numbers_or_subtitles() {
        assert_eq!(
            strip_editions(&normalize("Título 2 Deluxe")).base,
            "titulo 2"
        );
        assert_eq!(
            strip_editions(&normalize("Título: Prologue")).base,
            "titulo prologue"
        );
    }
}
