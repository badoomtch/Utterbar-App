//! Words → digits post-processing.
//!
//! ASR models ("Parakeet", "Whisper", etc.) emit numbers as words by
//! default — "twenty three", "one hundred and forty seven". Most
//! dictation users want digits. This module converts unambiguous
//! numeric phrases to digits while leaving idioms intact.
//!
//! Conservative philosophy:
//!   • Multi-word number spans (two or more contiguous number-words)
//!     are always converted: "twenty three" → "23".
//!   • Standalone low words ("one", "two", … "ten") are NOT converted
//!     by default — they're often idiomatic: "one of these days",
//!     "two ways to look at it". The cost of a false positive ("1 of
//!     these days") is much higher than the cost of a missed digit.
//!   • A standalone number IS converted when it's clearly numeric
//!     from immediate context: followed by a unit ("five pounds",
//!     "ten percent", "three o'clock"), preceded by a numeric anchor
//!     ("page seven", "chapter three", "step one"), or surrounding a
//!     digit-attachable suffix ("third" → "3rd").
//!
//! Out of scope (deliberate v1):
//!   • Decimals via "point" ("three point one four" → "3.14"). We
//!     handle integer phrases only.
//!   • Currency symbol injection. "five pounds" → "5 pounds", not
//!     "£5", because £/$/€ disambiguation needs locale.
//!   • Phone numbers / years spelled "nineteen ninety-nine".
//!
//! These can land in v2 once the basic transform proves itself.

use std::sync::OnceLock;

/// Public entry. Returns the input unchanged when no conversion happens
/// — caller can compare references for cheap idempotency checks.
pub fn words_to_digits(input: &str) -> String {
    if input.is_empty() {
        return input.to_string();
    }
    Converter::new().run(input)
}

struct Converter;

#[derive(Debug, Clone, Copy)]
enum Tok {
    /// A number word that contributes to a numeric phrase.
    NumWord(NumKind),
    /// A non-number word (anything else alphabetic).
    Word,
    /// Whitespace, punctuation, or anything else literal we preserve.
    Other,
}

#[derive(Debug, Clone, Copy)]
enum NumKind {
    /// 0..=9 — only converted when context demands it.
    Unit(u64),
    /// 10..=19 — same caution as Unit.
    Teen(u64),
    /// 20, 30, ..., 90.
    Ten(u64),
    /// "hundred" — multiplier (100).
    Hundred,
    /// "thousand", "million", "billion" — large multipliers.
    Big(u64),
    /// "and" — only acts as a number-word when it sits between number tokens.
    And,
    /// Ordinal forms like "third", "twenty-third". Carry the cardinal
    /// value + a suffix (`st`/`nd`/`rd`/`th`).
    Ordinal(u64, &'static str),
}

impl Converter {
    fn new() -> Self {
        Self
    }

    fn run(&self, input: &str) -> String {
        // Tokenise, scan for number-word runs, replace whole runs.
        // We tokenise on word boundaries while preserving whitespace
        // and punctuation as Other tokens so the output reads identically
        // outside the converted spans.
        let tokens = tokenise(input);
        let mut out = String::with_capacity(input.len());
        let mut i = 0;
        while i < tokens.len() {
            // Skip non-number tokens verbatim.
            let (kind, slice) = tokens[i];
            match kind {
                Tok::NumWord(_) => {
                    // Try to grow a numeric run starting here.
                    let (run_end, value, suffix) = scan_number_run(&tokens, i);
                    // run_end is the exclusive end of the run.
                    let run_len = run_end - i;
                    if run_len == 0 {
                        // Defensive — shouldn't happen.
                        out.push_str(slice);
                        i += 1;
                        continue;
                    }
                    if should_convert(&tokens, i, run_end, run_len) {
                        out.push_str(&format!("{}{}", value, suffix));
                        i = run_end;
                    } else {
                        // Leave the entire run as-is.
                        for tok in &tokens[i..run_end] {
                            out.push_str(tok.1);
                        }
                        i = run_end;
                    }
                }
                _ => {
                    out.push_str(slice);
                    i += 1;
                }
            }
        }
        out
    }
}

/// Tokenise the input into (kind, original-slice) pairs. Original
/// slices are non-overlapping and concatenate back to the input.
fn tokenise(input: &str) -> Vec<(Tok, &str)> {
    let mut out: Vec<(Tok, &str)> = Vec::new();
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        // Word: ASCII alphabetic + apostrophes, with optional internal
        // hyphens ("twenty-three" stays one token). Non-ASCII is
        // treated char-by-char as Other so the conversion never
        // accidentally swallows non-Latin characters from Whisper.
        if b.is_ascii_alphabetic() {
            let start = i;
            while i < bytes.len()
                && (bytes[i].is_ascii_alphabetic() || bytes[i] == b'-' || bytes[i] == b'\'')
            {
                i += 1;
            }
            let slice = &input[start..i];
            // Hyphenated number-words ("twenty-three", "thirty-second")
            // split on '-' for kind classification — but the slice
            // stays intact in the output.
            let kind = classify_word_or_phrase(slice);
            out.push((kind, slice));
            continue;
        }
        // Whitespace run.
        if b.is_ascii_whitespace() {
            let start = i;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            out.push((Tok::Other, &input[start..i]));
            continue;
        }
        // Single byte (punctuation, digit, non-ASCII byte).
        let start = i;
        i += 1;
        // Greedy run of identical-class non-alpha non-space bytes is
        // overkill; one byte per Other is fine.
        out.push((Tok::Other, &input[start..i]));
    }
    out
}

/// Classify a single hyphenated phrase ("twenty-three" → if BOTH halves
/// are number-words, it's a NumWord with combined value; otherwise it's
/// a regular Word).
fn classify_word_or_phrase(s: &str) -> Tok {
    if s.contains('-') {
        // Hyphenated. If every part is a recognised number-word, treat
        // the whole hyphenated phrase as one NumWord with the summed
        // value. Otherwise, plain Word.
        let mut total: u64 = 0;
        let mut ordinal_suffix: Option<&'static str> = None;
        for part in s.split('-') {
            let lower = part.to_ascii_lowercase();
            if let Some(kind) = match_number_word(&lower) {
                match kind {
                    NumKind::Unit(v) | NumKind::Teen(v) | NumKind::Ten(v) => total += v,
                    NumKind::Ordinal(v, suffix) => {
                        total += v;
                        ordinal_suffix = Some(suffix);
                    }
                    _ => return Tok::Word,
                }
            } else {
                return Tok::Word;
            }
        }
        if let Some(suffix) = ordinal_suffix {
            return Tok::NumWord(NumKind::Ordinal(total, suffix));
        }
        // Treat the combined value as a Ten-class (it's >=20 < 100).
        return Tok::NumWord(NumKind::Ten(total));
    }
    let lower = s.to_ascii_lowercase();
    match match_number_word(&lower) {
        Some(kind) => Tok::NumWord(kind),
        None => Tok::Word,
    }
}

/// Map a single lowercase word to its number-kind, or None.
fn match_number_word(w: &str) -> Option<NumKind> {
    static MAP: OnceLock<&'static [(&'static str, NumKind)]> = OnceLock::new();
    let map = MAP.get_or_init(|| {
        &[
            ("zero", NumKind::Unit(0)),
            ("one", NumKind::Unit(1)),
            ("two", NumKind::Unit(2)),
            ("three", NumKind::Unit(3)),
            ("four", NumKind::Unit(4)),
            ("five", NumKind::Unit(5)),
            ("six", NumKind::Unit(6)),
            ("seven", NumKind::Unit(7)),
            ("eight", NumKind::Unit(8)),
            ("nine", NumKind::Unit(9)),
            ("ten", NumKind::Teen(10)),
            ("eleven", NumKind::Teen(11)),
            ("twelve", NumKind::Teen(12)),
            ("thirteen", NumKind::Teen(13)),
            ("fourteen", NumKind::Teen(14)),
            ("fifteen", NumKind::Teen(15)),
            ("sixteen", NumKind::Teen(16)),
            ("seventeen", NumKind::Teen(17)),
            ("eighteen", NumKind::Teen(18)),
            ("nineteen", NumKind::Teen(19)),
            ("twenty", NumKind::Ten(20)),
            ("thirty", NumKind::Ten(30)),
            ("forty", NumKind::Ten(40)),
            ("fifty", NumKind::Ten(50)),
            ("sixty", NumKind::Ten(60)),
            ("seventy", NumKind::Ten(70)),
            ("eighty", NumKind::Ten(80)),
            ("ninety", NumKind::Ten(90)),
            ("hundred", NumKind::Hundred),
            ("thousand", NumKind::Big(1_000)),
            ("million", NumKind::Big(1_000_000)),
            ("billion", NumKind::Big(1_000_000_000)),
            ("and", NumKind::And),
            // Ordinals (cardinal value + display suffix).
            ("first", NumKind::Ordinal(1, "st")),
            ("second", NumKind::Ordinal(2, "nd")),
            ("third", NumKind::Ordinal(3, "rd")),
            ("fourth", NumKind::Ordinal(4, "th")),
            ("fifth", NumKind::Ordinal(5, "th")),
            ("sixth", NumKind::Ordinal(6, "th")),
            ("seventh", NumKind::Ordinal(7, "th")),
            ("eighth", NumKind::Ordinal(8, "th")),
            ("ninth", NumKind::Ordinal(9, "th")),
            ("tenth", NumKind::Ordinal(10, "th")),
            ("eleventh", NumKind::Ordinal(11, "th")),
            ("twelfth", NumKind::Ordinal(12, "th")),
            ("thirteenth", NumKind::Ordinal(13, "th")),
            ("fourteenth", NumKind::Ordinal(14, "th")),
            ("fifteenth", NumKind::Ordinal(15, "th")),
            ("sixteenth", NumKind::Ordinal(16, "th")),
            ("seventeenth", NumKind::Ordinal(17, "th")),
            ("eighteenth", NumKind::Ordinal(18, "th")),
            ("nineteenth", NumKind::Ordinal(19, "th")),
            ("twentieth", NumKind::Ordinal(20, "th")),
            ("thirtieth", NumKind::Ordinal(30, "th")),
            ("fortieth", NumKind::Ordinal(40, "th")),
            ("fiftieth", NumKind::Ordinal(50, "th")),
            ("sixtieth", NumKind::Ordinal(60, "th")),
            ("seventieth", NumKind::Ordinal(70, "th")),
            ("eightieth", NumKind::Ordinal(80, "th")),
            ("ninetieth", NumKind::Ordinal(90, "th")),
        ]
    });
    map.iter()
        .find_map(|(k, v)| if *k == w { Some(*v) } else { None })
}

/// Walk forward from `start` to find the longest valid numeric run.
/// Returns (exclusive_end, value, ordinal_suffix). When the run is a
/// single non-ordinal Unit/Teen with no surrounding numeric context,
/// the caller still uses `should_convert` to decide whether to apply.
fn scan_number_run(tokens: &[(Tok, &str)], start: usize) -> (usize, u64, &'static str) {
    let mut total: u64 = 0;
    let mut current: u64 = 0;
    let mut ordinal_suffix = "";
    let mut i = start;
    let mut last_useful = start;

    // Grammar of the current additive group (the sub-hundred part being
    // assembled). Without this, ANY run of number-words was summed —
    // "twenty thirty" became "50", "five six" became "11" — but those are
    // two separate numbers spoken back to back, not one composed number.
    // Only genuine compositions ("twenty three" = 23) should merge.
    //   • `group_has_tens` — a tens word (20..90) is in the group; it may
    //     still accept ONE trailing unit ("twenty" + "three" = 23).
    //   • `group_has_ones` — a units/teens word (1..19) fills the ones
    //     place; the group is complete, nothing more can be added.
    // A hundred/thousand/million multiplier closes the group and clears
    // both flags so a fresh sub-hundred group can follow
    // ("one hundred twenty three").
    let mut group_has_tens = false;
    let mut group_has_ones = false;

    while i < tokens.len() {
        match tokens[i].0 {
            Tok::NumWord(NumKind::Unit(v)) => {
                // A unit only extends the group right after a bare tens
                // word ("twenty three"). After a unit or teen already
                // placed, it's the start of a NEW number — end the run.
                if group_has_ones {
                    break;
                }
                current += v;
                group_has_ones = true;
                last_useful = i + 1;
                i += 1;
            }
            Tok::NumWord(NumKind::Teen(v)) => {
                // Teens occupy both the tens and ones place, so they can't
                // follow another additive word in the same group
                // ("thirteen fourteen" is two numbers).
                if group_has_tens || group_has_ones {
                    break;
                }
                current += v;
                group_has_ones = true;
                last_useful = i + 1;
                i += 1;
            }
            Tok::NumWord(NumKind::Ten(v)) => {
                // A tens word can't follow another tens/ones word in the
                // same group ("twenty thirty", "five sixty").
                if group_has_tens || group_has_ones {
                    break;
                }
                current += v;
                group_has_tens = true;
                last_useful = i + 1;
                i += 1;
            }
            Tok::NumWord(NumKind::Hundred) => {
                if current == 0 {
                    current = 100;
                } else {
                    current *= 100;
                }
                group_has_tens = false;
                group_has_ones = false;
                last_useful = i + 1;
                i += 1;
            }
            Tok::NumWord(NumKind::Big(mult)) => {
                if current == 0 {
                    current = mult;
                } else {
                    current *= mult;
                }
                total += current;
                current = 0;
                group_has_tens = false;
                group_has_ones = false;
                last_useful = i + 1;
                i += 1;
            }
            Tok::NumWord(NumKind::And) => {
                // "and" is only a number-token when it joins the
                // hundreds/thousands part of a single number to its
                // tens-and-units tail ("one hundred and forty-seven",
                // "one thousand and one"). A bare "one and two" is a
                // list, not a number — the previous guard
                // (`current > 0 || total > 0`) was too loose and made
                // the converter sum the operands into "3". Only allow
                // the conjunction when the run has already crossed a
                // hundred-or-bigger boundary: either `current >= 100`
                // (saw a Hundred) or `total > 0` (saw a Big multiplier
                // which always rolls current into total at >= 1000).
                let mut k = i + 1;
                while k < tokens.len() {
                    if matches!(tokens[k].0, Tok::Other) {
                        k += 1;
                        continue;
                    }
                    break;
                }
                let next_is_num = k < tokens.len()
                    && matches!(tokens[k].0, Tok::NumWord(n) if !matches!(n, NumKind::And));
                if next_is_num && (current >= 100 || total > 0) {
                    i += 1; // consume the "and", carry on.
                } else {
                    break;
                }
            }
            Tok::NumWord(NumKind::Ordinal(v, suffix)) => {
                // Ordinal is always terminal. Either standalone
                // ("third") or finishing a run ("twenty-third" handled
                // by classify_word_or_phrase as a single Ordinal token).
                current += v;
                ordinal_suffix = suffix;
                last_useful = i + 1;
                break;
            }
            Tok::Other => {
                // Whitespace inside a run is fine; punctuation breaks it.
                if tokens[i].1.chars().all(|c| c.is_ascii_whitespace()) {
                    i += 1;
                    continue;
                }
                break;
            }
            Tok::Word => break,
        }
    }

    let value = total + current;
    (last_useful, value, ordinal_suffix)
}

/// Decide whether to actually apply the conversion. Multi-token runs
/// always convert. Single Unit/Teen runs need contextual evidence.
fn should_convert(tokens: &[(Tok, &str)], start: usize, end: usize, word_count: usize) -> bool {
    // Multi-word runs are unambiguous.
    if word_count > 1 {
        return true;
    }
    // Ordinals are unambiguous ("third", "twentieth").
    if matches!(tokens[start].0, Tok::NumWord(NumKind::Ordinal(_, _))) {
        return true;
    }
    // Tens (twenty, thirty, ...) and Hundred standalone — convert.
    // (Catches "I had twenty" etc.)
    if matches!(
        tokens[start].0,
        Tok::NumWord(NumKind::Ten(_))
            | Tok::NumWord(NumKind::Hundred)
            | Tok::NumWord(NumKind::Big(_))
    ) {
        return true;
    }
    // Teens (eleven .. nineteen) standalone — convert. "Eleven" rarely
    // means anything other than the number 11.
    if matches!(tokens[start].0, Tok::NumWord(NumKind::Teen(_))) {
        return true;
    }
    // Unit (zero..nine) standalone. Convert only when context is clearly
    // numeric.
    let prev = prev_word_lower(tokens, start);
    let next = next_word_lower(tokens, end);
    if let Some(p) = prev.as_deref() {
        if NUMERIC_ANCHORS_BEFORE.contains(&p) {
            return true;
        }
    }
    if let Some(n) = next.as_deref() {
        if NUMERIC_ANCHORS_AFTER.contains(&n) {
            return true;
        }
    }
    false
}

const NUMERIC_ANCHORS_BEFORE: &[&str] = &[
    "page", "chapter", "section", "step", "room", "version", "v", "level", "tier", "table", "row",
    "column", "track", "episode", "issue", "volume", "rule", "item", "no", "number", "no.",
];

const NUMERIC_ANCHORS_AFTER: &[&str] = &[
    "pounds",
    "pound",
    "dollars",
    "dollar",
    "euros",
    "euro",
    "pence",
    "cents",
    "percent",
    "%",
    "kilos",
    "kilo",
    "kilograms",
    "kilogram",
    "kg",
    "grams",
    "gram",
    "g",
    "metres",
    "meters",
    "metre",
    "meter",
    "m",
    "kilometres",
    "kilometers",
    "km",
    "miles",
    "mile",
    "minutes",
    "minute",
    "seconds",
    "second",
    "hours",
    "hour",
    "days",
    "day",
    "weeks",
    "week",
    "months",
    "month",
    "years",
    "year",
    "o'clock",
    "am",
    "pm",
];

fn prev_word_lower(tokens: &[(Tok, &str)], idx: usize) -> Option<String> {
    let mut i = idx;
    while i > 0 {
        i -= 1;
        match tokens[i].0 {
            Tok::Word | Tok::NumWord(_) => return Some(tokens[i].1.to_ascii_lowercase()),
            _ => continue,
        }
    }
    None
}

fn next_word_lower(tokens: &[(Tok, &str)], idx: usize) -> Option<String> {
    let mut i = idx;
    while i < tokens.len() {
        match tokens[i].0 {
            Tok::Word | Tok::NumWord(_) => return Some(tokens[i].1.to_ascii_lowercase()),
            _ => i += 1,
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(input: &str) -> String {
        words_to_digits(input)
    }

    #[test]
    fn multi_word_runs_convert() {
        assert_eq!(t("twenty three apples"), "23 apples");
        assert_eq!(t("twenty-three apples"), "23 apples");
        assert_eq!(t("one hundred and forty seven"), "147");
        assert_eq!(t("three thousand"), "3000");
        assert_eq!(t("two million"), "2000000");
    }

    #[test]
    fn standalone_low_words_left_alone() {
        // Idiom protection — these must stay as words.
        assert_eq!(t("one of these days"), "one of these days");
        assert_eq!(t("two ways to look"), "two ways to look");
        assert_eq!(t("three of them"), "three of them");
    }

    #[test]
    fn standalone_low_words_with_unit_convert() {
        assert_eq!(t("five pounds"), "5 pounds");
        assert_eq!(t("ten percent"), "10 percent");
        assert_eq!(t("three o'clock"), "3 o'clock");
    }

    #[test]
    fn standalone_low_words_with_anchor_convert() {
        assert_eq!(t("page seven"), "page 7");
        assert_eq!(t("chapter three"), "chapter 3");
        assert_eq!(t("step one"), "step 1");
    }

    #[test]
    fn ordinals_convert() {
        assert_eq!(t("the third one"), "the 3rd one");
        assert_eq!(t("twenty-third of June"), "23rd of June");
    }

    #[test]
    fn tens_and_teens_convert_standalone() {
        assert_eq!(t("I had twenty"), "I had 20");
        assert_eq!(t("about fifteen"), "about 15");
    }

    #[test]
    fn punctuation_breaks_run() {
        assert_eq!(t("twenty, three"), "20, three");
    }

    #[test]
    fn empty_passthrough() {
        assert_eq!(t(""), "");
        assert_eq!(t("hello world"), "hello world");
    }

    /// Regression: two separate numbers spoken back to back must NOT be
    /// summed into one. Before the grammar-aware scanner, any contiguous
    /// run of number-words was added together — "twenty thirty" → "50",
    /// "five six" → "11" — which is wrong. Only genuine compositions
    /// ("twenty three" = 23) merge; anything that isn't a valid single
    /// number breaks into separate tokens.
    #[test]
    fn back_to_back_numbers_do_not_sum() {
        // Two tens words: two numbers, not their sum.
        assert_eq!(t("twenty thirty"), "20 30");
        assert_eq!(t("fifty sixty"), "50 60");
        // Tens then unit IS a valid composition and still merges.
        assert_eq!(t("twenty three"), "23");
        assert_eq!(t("forty five"), "45");
        // Two teens: two numbers.
        assert_eq!(t("thirteen fourteen"), "13 14");
        // Ten then teen: not composable.
        assert_eq!(t("twenty thirteen"), "20 13");
        // Low units back to back stay as words (idiom-safe) and are never
        // summed.
        assert_eq!(t("five six"), "five six");
        // Composed hundreds still work after the group-reset on the
        // multiplier.
        assert_eq!(t("one hundred twenty three"), "123");
        assert_eq!(t("three hundred forty"), "340");
    }

    /// Regression: "one and two" must NOT collapse to "3". "and" is only
    /// a number-conjunction once the run has crossed a hundred-or-bigger
    /// boundary; below that it's an English connective between two
    /// separate things. "one hundred and forty seven" (covered above)
    /// must keep working.
    #[test]
    fn bare_and_between_low_numbers_does_not_sum() {
        assert_eq!(t("one and two"), "one and two");
        assert_eq!(t("three and four"), "three and four");
        assert_eq!(t("twenty and thirty"), "20 and 30");
        // Mixed: the second number still digit-ises via its own anchor
        // or context, the "and" stays literal.
        assert_eq!(t("page one and page two"), "page 1 and page 2");
        // Hundreds-and form still works.
        assert_eq!(t("one hundred and two"), "102");
        assert_eq!(t("one thousand and one"), "1001");
    }
}
