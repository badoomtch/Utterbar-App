//! Output validation for ASR transcripts before they reach the user.
//!
//! Pure string-level checks. No I/O, no audio, no async. Sits between
//! the ASR engine + post-processing pipeline (vocab replacements, LLM
//! rewrite, dialect normalisation, auto-capitalise, etc.) and the
//! split that text-injects, writes a history row, and emits Success.
//!
//! Engine-internal filters (sherpa_whisper::is_whisper_hallucination
//! and sherpa_parakeet's filler-word gate) stay where they are. This
//! module is the second line of defence: it catches the cases the
//! engine internals don't, and centralises the decision point so a
//! single drop verdict drives both the no-text path and the future
//! Meeting Mode / file-transcription wiring.
//!
//! Conservative by design. The cost of dropping a valid transcript is
//! much higher than the cost of letting a marginally-bad one through,
//! because the user can always re-dictate but cannot recover a
//! silently-dropped sentence.

use serde::Serialize;

/// Engine family the recogniser belongs to. Matters for the
/// foreign-script rule, which is stricter on English-only engines
/// (Parakeet) than on multilingual ones (Whisper).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineFamily {
    /// Parakeet TDT - English-only, hard-pinned. Foreign-script
    /// output here is always a hallucination.
    Parakeet,
    /// Whisper - multilingual. Foreign-script output may be
    /// legitimate depending on `LanguagePin`.
    Whisper,
}

/// What the user has configured the recogniser language to. Drives
/// whether foreign-script output is treated as a hallucination or
/// expected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LanguagePin {
    /// Engine has no language config (Parakeet) or user chose
    /// English explicitly. Foreign script is unexpected.
    EnglishOnly,
    /// Whisper auto-detect. Caller has opted into multilingual
    /// behaviour; foreign script is legitimate.
    Auto,
    /// Whisper pinned to a specific non-English language (e.g.
    /// `"es"`). Output in that language's script is legitimate;
    /// only output in some other unrelated script would be
    /// suspicious. We don't try to enforce that - the existing
    /// engine pin already does the heavy lifting - so this case
    /// behaves the same as `Auto` for validation purposes.
    OtherExplicit,
}

/// Inputs the validator needs alongside the candidate text.
#[derive(Debug, Clone)]
pub struct ValidationContext {
    pub engine: EngineFamily,
    pub language_pin: LanguagePin,
    /// Wall-clock duration of the captured audio. Used so the
    /// repeated-token rule can skip very short clips where short
    /// natural repetition ("no no no" in 0.5 s) is plausible - not
    /// currently load-bearing because the rule's threshold (>=6
    /// consecutive tokens) is already conservative, but kept on the
    /// context for forward-compat with rules that want it.
    pub audio_duration_ms: u32,
}

/// Verdict on a candidate transcript.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationVerdict {
    /// Pass through to text injection / history insertion.
    Accept,
    /// Drop. `user_notice` distinguishes "non-event, treat like
    /// silence" (false) from "we filtered something - tell the user"
    /// (true). The caller maps this to the floating-bar pill: false
    /// reuses the existing `No audio detected` UX; true surfaces a
    /// concise notice on the same NoAudio phase.
    Drop {
        reason: DropReason,
        user_notice: bool,
    },
}

/// Why the validator declined a transcript. Goes into structured
/// logs so soak telemetry shows which rules fire in the wild.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DropReason {
    /// Empty string after the engine and post-processing.
    Empty,
    /// Whitespace-only after trim.
    WhitespaceOnly,
    /// A single punctuation character or similar nub (e.g. `"."`).
    /// Whisper occasionally returns just a period on silence.
    SinglePunctuation,
    /// Same alphabetic token repeated 6+ times in a row - a
    /// classic Whisper decoder loop.
    RepeatedToken,
    /// 10+ consecutive identical punctuation characters.
    RepeatedPunctuation,
    /// 80% or more of the non-ASCII letters belong to a single
    /// non-Latin script (CJK, Cyrillic, Arabic, Devanagari, Hangul)
    /// on a path where the user has explicitly chosen English.
    /// Almost always a Whisper auto-detect mis-classification or a
    /// hallucination.
    UnexpectedForeignScript,
}

impl DropReason {
    /// Stable lower-snake string suitable for tracing fields.
    pub fn as_str(self) -> &'static str {
        match self {
            DropReason::Empty => "empty",
            DropReason::WhitespaceOnly => "whitespace_only",
            DropReason::SinglePunctuation => "single_punctuation",
            DropReason::RepeatedToken => "repeated_token",
            DropReason::RepeatedPunctuation => "repeated_punctuation",
            DropReason::UnexpectedForeignScript => "unexpected_foreign_script",
        }
    }
}

/// Validate an ASR transcript. Returns `Accept` to continue the
/// success path, or `Drop { reason, user_notice }` to short-circuit
/// before text injection and history insertion.
///
/// First-fail wins. Rules are listed in widening order: the cheapest
/// and most obvious checks first, then the rules that need scanning
/// the string. Order matters for the diagnostic - a string that's
/// both empty and (vacuously) "all whitespace" reports `Empty`.
pub fn validate(text: &str, ctx: &ValidationContext) -> ValidationVerdict {
    if text.is_empty() {
        return drop_quiet(DropReason::Empty);
    }
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return drop_quiet(DropReason::WhitespaceOnly);
    }

    // Single punctuation-or-symbol char as the entire output. Catches
    // ".", "?", a lone dash, a lone music glyph, etc. Letters fall
    // through (so single-letter legitimate output like "I" or "a"
    // still passes).
    if trimmed.chars().count() == 1 {
        let c = trimmed.chars().next().unwrap();
        if !c.is_alphanumeric() {
            return drop_quiet(DropReason::SinglePunctuation);
        }
    }

    // Repeated alphabetic token (Whisper decoder loop). Tokens are
    // whitespace-separated, normalised to lowercase + stripped of
    // surrounding punctuation so "the, the, the the the the" is
    // counted as the same token. Threshold is 6 consecutive - well
    // above the longest natural repetition we'd expect ("no no no
    // no" five times feels manic but isn't a loop).
    if has_repeated_alpha_token(trimmed, 6) {
        return drop_noticed(DropReason::RepeatedToken);
    }

    // Repeated punctuation run. >=10 consecutive identical
    // punctuation chars anywhere in the string. Catches
    // "!!!!!!!!!!!!!!" and ".................." outputs that
    // sometimes leak from the Whisper decoder.
    if has_repeated_punctuation(trimmed, 10) {
        return drop_noticed(DropReason::RepeatedPunctuation);
    }

    // Foreign-script gate. Only fires when the caller has opted
    // into English-only. Auto-detect and explicit non-English pins
    // pass through.
    if matches!(ctx.language_pin, LanguagePin::EnglishOnly) && unexpected_foreign_script(trimmed) {
        return drop_noticed(DropReason::UnexpectedForeignScript);
    }

    // Note on `engine` and `audio_duration_ms`: these are on the
    // ValidationContext for forward-compat with rules that want
    // them (e.g. a future engine-conditional filler-word rule) but
    // not consulted by any rule in this first patch. The
    // language-pin gate above already discriminates Parakeet (which
    // the caller maps to LanguagePin::EnglishOnly) from multilingual
    // Whisper.
    let _ = ctx.engine;
    let _ = ctx.audio_duration_ms;

    ValidationVerdict::Accept
}

fn drop_quiet(reason: DropReason) -> ValidationVerdict {
    ValidationVerdict::Drop {
        reason,
        user_notice: false,
    }
}

fn drop_noticed(reason: DropReason) -> ValidationVerdict {
    ValidationVerdict::Drop {
        reason,
        user_notice: true,
    }
}

/// True if any alphabetic token (lowercase, edge-punctuation
/// stripped) repeats `threshold` times consecutively. Pure
/// punctuation tokens are skipped.
fn has_repeated_alpha_token(text: &str, threshold: usize) -> bool {
    let mut prev: Option<String> = None;
    let mut run: usize = 0;
    for raw in text.split_whitespace() {
        let key: String = raw
            .trim_matches(|c: char| !c.is_alphanumeric())
            .to_lowercase();
        if key.is_empty() || !key.chars().any(|c| c.is_alphabetic()) {
            // Skip pure-punctuation or pure-numeric tokens - the
            // rule targets word-level decoder loops, not sequences
            // of "..." or digit runs.
            prev = None;
            run = 0;
            continue;
        }
        if Some(&key) == prev.as_ref() {
            run += 1;
        } else {
            run = 1;
            prev = Some(key);
        }
        if run >= threshold {
            return true;
        }
    }
    false
}

/// True if any punctuation char (in the Unicode "punctuation or
/// symbol" sense) repeats `threshold` times in a row anywhere in
/// the string.
fn has_repeated_punctuation(text: &str, threshold: usize) -> bool {
    let mut prev: Option<char> = None;
    let mut run: usize = 0;
    for c in text.chars() {
        let is_punct = !c.is_alphanumeric() && !c.is_whitespace();
        if is_punct && Some(c) == prev {
            run += 1;
            if run >= threshold {
                return true;
            }
        } else if is_punct {
            run = 1;
            prev = Some(c);
        } else {
            run = 0;
            prev = None;
        }
    }
    false
}

/// True if >=80% of the *non-ASCII letter* characters in `text`
/// belong to a single non-Latin script we recognise (CJK,
/// Cyrillic, Arabic, Devanagari, Hangul). ASCII letters and
/// non-letter characters (digits, punctuation, currency symbols,
/// whitespace) are ignored entirely.
///
/// The "non-ASCII letter" denominator is what makes this safe for
/// English text peppered with currency symbols, the degree sign,
/// accented French/German vowels, or Greek letters in code: those
/// are non-ASCII but not in any of the listed script blocks, so
/// they don't push a non-Latin script over the 80% line.
fn unexpected_foreign_script(text: &str) -> bool {
    let mut non_latin_letters: usize = 0;
    let mut total_non_ascii_letters: usize = 0;
    let mut dominant_script_count: usize = 0;
    let mut counts = [0usize; 5]; // [cjk, cyrillic, arabic, devanagari, hangul]

    for c in text.chars() {
        if !c.is_alphabetic() {
            continue;
        }
        if c.is_ascii() {
            continue;
        }
        total_non_ascii_letters += 1;
        if let Some(idx) = non_latin_script_index(c) {
            non_latin_letters += 1;
            counts[idx] += 1;
            if counts[idx] > dominant_script_count {
                dominant_script_count = counts[idx];
            }
        }
    }

    // Need at least a few characters to make a confident call -
    // a stray Greek letter shouldn't trip the rule on an
    // otherwise-English sentence. The threshold is total non-ASCII
    // letters, not the string length, so short pure-foreign output
    // (a two-character CJK word) still qualifies (2 >= 2).
    if total_non_ascii_letters < 2 {
        return false;
    }
    // 80% of non-ASCII letters belong to one non-Latin script.
    dominant_script_count * 5 >= total_non_ascii_letters * 4
        && dominant_script_count == non_latin_letters
}

/// Map a character to a non-Latin script bucket index, or None if
/// it doesn't fall in any tracked block.
///
/// We deliberately exclude Latin-extended (accented French/German/
/// Spanish vowels), Greek (one-off identifiers in code), IPA, and
/// general-punctuation/symbol blocks. The rule is "is this output
/// dominantly in a script that English speakers don't normally
/// produce by accident", not "does this contain any non-ASCII".
fn non_latin_script_index(c: char) -> Option<usize> {
    let cp = c as u32;
    // CJK Unified Ideographs + Hiragana + Katakana + CJK Symbols.
    // Covers Chinese, Japanese kana/kanji, common CJK punctuation
    // when it carries letter semantics. Compatibility blocks
    // included so half-width katakana and CJK Compat Ideographs
    // count.
    if (0x3040..=0x30FF).contains(&cp)        // Hiragana + Katakana
        || (0x31F0..=0x31FF).contains(&cp)    // Katakana Phonetic Extensions
        || (0x3400..=0x4DBF).contains(&cp)    // CJK Unified Ideographs Ext A
        || (0x4E00..=0x9FFF).contains(&cp)    // CJK Unified Ideographs
        || (0xF900..=0xFAFF).contains(&cp)    // CJK Compatibility Ideographs
        || (0xFF66..=0xFF9F).contains(&cp)
    {
        return Some(0);
    }
    // Cyrillic + Cyrillic Supplement.
    if (0x0400..=0x04FF).contains(&cp) || (0x0500..=0x052F).contains(&cp) {
        return Some(1);
    }
    // Arabic + Arabic Supplement + Arabic Extended-A.
    if (0x0600..=0x06FF).contains(&cp)
        || (0x0750..=0x077F).contains(&cp)
        || (0x08A0..=0x08FF).contains(&cp)
    {
        return Some(2);
    }
    // Devanagari.
    if (0x0900..=0x097F).contains(&cp) {
        return Some(3);
    }
    // Hangul Syllables + Jamo.
    if (0xAC00..=0xD7AF).contains(&cp) || (0x1100..=0x11FF).contains(&cp) {
        return Some(4);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx_english_parakeet() -> ValidationContext {
        ValidationContext {
            engine: EngineFamily::Parakeet,
            language_pin: LanguagePin::EnglishOnly,
            audio_duration_ms: 4_000,
        }
    }

    fn ctx_whisper_auto() -> ValidationContext {
        ValidationContext {
            engine: EngineFamily::Whisper,
            language_pin: LanguagePin::Auto,
            audio_duration_ms: 4_000,
        }
    }

    fn ctx_whisper_english() -> ValidationContext {
        ValidationContext {
            engine: EngineFamily::Whisper,
            language_pin: LanguagePin::EnglishOnly,
            audio_duration_ms: 4_000,
        }
    }

    fn ctx_whisper_other() -> ValidationContext {
        ValidationContext {
            engine: EngineFamily::Whisper,
            language_pin: LanguagePin::OtherExplicit,
            audio_duration_ms: 4_000,
        }
    }

    fn assert_drop(v: ValidationVerdict, expected: DropReason, expected_notice: bool) {
        match v {
            ValidationVerdict::Drop {
                reason,
                user_notice,
            } => {
                assert_eq!(reason, expected, "wrong drop reason");
                assert_eq!(
                    user_notice, expected_notice,
                    "wrong notice flag for {expected:?}"
                );
            }
            ValidationVerdict::Accept => panic!("expected Drop({expected:?}), got Accept"),
        }
    }

    #[test]
    fn empty_drops_quietly() {
        assert_drop(
            validate("", &ctx_english_parakeet()),
            DropReason::Empty,
            false,
        );
    }

    #[test]
    fn whitespace_only_drops_quietly() {
        assert_drop(
            validate("   \n\t ", &ctx_english_parakeet()),
            DropReason::WhitespaceOnly,
            false,
        );
    }

    #[test]
    fn lone_period_drops_quietly() {
        assert_drop(
            validate(".", &ctx_english_parakeet()),
            DropReason::SinglePunctuation,
            false,
        );
        assert_drop(
            validate("?", &ctx_english_parakeet()),
            DropReason::SinglePunctuation,
            false,
        );
        // Padded with whitespace - trim happens first.
        assert_drop(
            validate("   .   ", &ctx_english_parakeet()),
            DropReason::SinglePunctuation,
            false,
        );
    }

    #[test]
    fn lone_letter_passes() {
        // "I" and "a" are valid one-character utterances.
        assert_eq!(
            validate("I", &ctx_english_parakeet()),
            ValidationVerdict::Accept
        );
        assert_eq!(
            validate("a", &ctx_english_parakeet()),
            ValidationVerdict::Accept
        );
    }

    #[test]
    fn repeated_token_drops_with_notice() {
        assert_drop(
            validate("the the the the the the", &ctx_english_parakeet()),
            DropReason::RepeatedToken,
            true,
        );
        // Same token with edge punctuation still counts.
        assert_drop(
            validate("the, the, the. the the the", &ctx_english_parakeet()),
            DropReason::RepeatedToken,
            true,
        );
        // Embedded in longer output.
        assert_drop(
            validate(
                "ok so the the the the the the and then we",
                &ctx_english_parakeet(),
            ),
            DropReason::RepeatedToken,
            true,
        );
    }

    #[test]
    fn short_repetition_passes() {
        // Five repeats - below the 6-token threshold. The validator
        // shouldn't fire on natural emphatic speech.
        assert_eq!(
            validate("no no no", &ctx_english_parakeet()),
            ValidationVerdict::Accept
        );
        assert_eq!(
            validate("ha ha ha ha", &ctx_english_parakeet()),
            ValidationVerdict::Accept
        );
        assert_eq!(
            validate("no no no no no", &ctx_english_parakeet()),
            ValidationVerdict::Accept
        );
    }

    #[test]
    fn repeated_punctuation_drops_with_notice() {
        assert_drop(
            validate("!!!!!!!!!!", &ctx_english_parakeet()),
            DropReason::RepeatedPunctuation,
            true,
        );
        assert_drop(
            validate("hello..........", &ctx_english_parakeet()),
            DropReason::RepeatedPunctuation,
            true,
        );
    }

    #[test]
    fn short_punctuation_passes() {
        assert_eq!(
            validate("really!!", &ctx_english_parakeet()),
            ValidationVerdict::Accept
        );
        assert_eq!(
            validate("wait...", &ctx_english_parakeet()),
            ValidationVerdict::Accept
        );
    }

    #[test]
    fn cjk_dropped_on_english_only() {
        assert_drop(
            validate("你好世界", &ctx_english_parakeet()),
            DropReason::UnexpectedForeignScript,
            true,
        );
        assert_drop(
            validate("こんにちは", &ctx_english_parakeet()),
            DropReason::UnexpectedForeignScript,
            true,
        );
    }

    #[test]
    fn cyrillic_dropped_on_english_only() {
        assert_drop(
            validate("Привет мир", &ctx_whisper_english()),
            DropReason::UnexpectedForeignScript,
            true,
        );
    }

    #[test]
    fn arabic_dropped_on_english_only() {
        assert_drop(
            validate("مرحبا بالعالم", &ctx_english_parakeet()),
            DropReason::UnexpectedForeignScript,
            true,
        );
    }

    #[test]
    fn cjk_passes_on_whisper_auto() {
        assert_eq!(
            validate("你好世界", &ctx_whisper_auto()),
            ValidationVerdict::Accept
        );
        assert_eq!(
            validate("こんにちは", &ctx_whisper_auto()),
            ValidationVerdict::Accept
        );
    }

    #[test]
    fn cjk_passes_on_explicit_non_english() {
        // User pinned Whisper to e.g. Chinese - chinese output is
        // exactly what they want.
        assert_eq!(
            validate("你好世界", &ctx_whisper_other()),
            ValidationVerdict::Accept
        );
    }

    #[test]
    fn currency_symbol_passes() {
        assert_eq!(
            validate("£39.99", &ctx_english_parakeet()),
            ValidationVerdict::Accept
        );
        assert_eq!(
            validate("That's €100, $5, and ¥50.", &ctx_english_parakeet()),
            ValidationVerdict::Accept
        );
    }

    #[test]
    fn short_commands_pass() {
        // The kind of one- or two-word utterances Utterbar users
        // do dictate - must never be filtered.
        for cmd in [
            "yes",
            "no",
            "send",
            "delete that",
            "reply",
            "ok",
            "stop",
            "open inbox",
        ] {
            assert_eq!(
                validate(cmd, &ctx_english_parakeet()),
                ValidationVerdict::Accept,
                "short command was filtered: {cmd}"
            );
        }
    }

    #[test]
    fn code_snippet_passes() {
        assert_eq!(
            validate("const x = useState();", &ctx_english_parakeet()),
            ValidationVerdict::Accept
        );
        // Greek letter in identifier - single non-Latin char in a
        // sea of ASCII shouldn't trip the foreign-script rule.
        assert_eq!(
            validate("let π = 3.14;", &ctx_english_parakeet()),
            ValidationVerdict::Accept
        );
    }

    #[test]
    fn accented_latin_passes_on_english_only() {
        // French / German / Spanish accented vowels are in
        // Latin-extended, NOT in any tracked non-Latin script
        // block, so they don't trip the rule even on EnglishOnly.
        assert_eq!(
            validate("café au lait", &ctx_english_parakeet()),
            ValidationVerdict::Accept
        );
        assert_eq!(
            validate("Über schön", &ctx_english_parakeet()),
            ValidationVerdict::Accept
        );
    }

    #[test]
    fn mixed_english_french_passes_on_english_only() {
        // Polyglot output where one phrase is French. Latin script
        // throughout - passes regardless of language pin.
        assert_eq!(
            validate(
                "Let's grab a café before the meeting starts.",
                &ctx_english_parakeet()
            ),
            ValidationVerdict::Accept
        );
    }

    #[test]
    fn drop_reason_log_strings_are_stable() {
        // Sanity - these strings appear in tracing fields and any
        // future log-greppers will rely on the snake_case form.
        assert_eq!(DropReason::Empty.as_str(), "empty");
        assert_eq!(DropReason::WhitespaceOnly.as_str(), "whitespace_only");
        assert_eq!(DropReason::SinglePunctuation.as_str(), "single_punctuation");
        assert_eq!(DropReason::RepeatedToken.as_str(), "repeated_token");
        assert_eq!(
            DropReason::RepeatedPunctuation.as_str(),
            "repeated_punctuation"
        );
        assert_eq!(
            DropReason::UnexpectedForeignScript.as_str(),
            "unexpected_foreign_script"
        );
    }
}
