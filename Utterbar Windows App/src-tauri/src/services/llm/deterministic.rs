//! Deterministic fallback post-processor. Always available; no model required.
//!
//! Handles the cases the UX must still feel good without a local LLM:
//!   - sentence-initial capitalisation
//!   - terminal punctuation
//!   - filler-word removal
//!   - whitespace normalisation

use anyhow::Result;
use async_trait::async_trait;

use super::provider::{LlmProvider, ProcContext};

pub struct DeterministicProvider;

const FILLERS: &[&str] = &[
    "um", "uh", "erm", "like", "you know", "i mean", "sort of", "kind of",
];

#[async_trait]
impl LlmProvider for DeterministicProvider {
    fn id(&self) -> &str {
        "deterministic"
    }
    async fn is_available(&self) -> bool {
        true
    }

    async fn rewrite(&self, input: &str, _system: &str, _ctx: &ProcContext) -> Result<String> {
        let cleaned = strip_fillers(input);
        let normalised = normalise_whitespace(&cleaned);
        let punctuated = ensure_terminal_punctuation(&normalised);
        Ok(capitalise_sentences(&punctuated))
    }
}

pub fn strip_fillers(text: &str) -> String {
    let mut out = text.to_string();
    for f in FILLERS {
        // word-boundary replacement, case-insensitive
        let pat = format!(r"(?i)\b{}\b[,\s]?", regex_escape(f));
        out = replace_regex(&out, &pat, "");
    }
    out
}

pub fn normalise_whitespace(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last_space = false;
    for ch in text.chars() {
        if ch.is_whitespace() {
            if !last_space {
                out.push(' ');
            }
            last_space = true;
        } else {
            out.push(ch);
            last_space = false;
        }
    }
    out.trim().to_string()
}

pub fn ensure_terminal_punctuation(text: &str) -> String {
    let t = text.trim();
    if t.is_empty() {
        return String::new();
    }
    let last = t.chars().last().unwrap();
    if matches!(last, '.' | '!' | '?' | ':' | ';' | ',') {
        t.to_string()
    } else {
        format!("{t}.")
    }
}

pub fn capitalise_sentences(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut capitalise_next = true;
    for ch in text.chars() {
        if capitalise_next && ch.is_alphabetic() {
            for c in ch.to_uppercase() {
                out.push(c);
            }
            capitalise_next = false;
        } else {
            out.push(ch);
            if matches!(ch, '.' | '!' | '?') {
                capitalise_next = true;
            }
        }
    }
    out
}

// --- tiny regex shim — avoid adding the full `regex` crate for this narrow use case ---

fn regex_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if matches!(
            c,
            '.' | '+' | '*' | '?' | '(' | ')' | '|' | '[' | ']' | '{' | '}' | '^' | '$' | '\\'
        ) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

// Delegates to the `regex` crate if available; otherwise does a best-effort literal replace.
// For v1 we do a case-insensitive literal word-boundary replace — good enough for a handful of fillers.
fn replace_regex(haystack: &str, _pattern: &str, _replacement: &str) -> String {
    // Without the regex dep, we do a pragmatic per-word scan.
    //
    // UTF-8 safety (v0.15.6 fix): the previous version of this loop
    // did `let lower = haystack.to_lowercase();` and then sliced
    // `&lower[idx..]` on every iteration where `idx` was a haystack
    // byte index. Two latent bugs there:
    //
    //   1. Multibyte chars like `£` (UTF-8 0xC2 0xA3) made `idx`
    //      land mid-char, and `lower[idx..]` panics with "byte
    //      index N is not a char boundary". Crash observed
    //      2026-04-29 on a Parakeet dictation containing `£200`.
    //   2. `to_lowercase` can change byte length (e.g. Turkish `İ`
    //      lowercases to `i\u{307}` — 2 bytes -> 3 bytes), so
    //      indexing `lower` with offsets derived from `haystack`
    //      drifted silently for non-ASCII content.
    //
    // Fix: drop the `lower` precomputation entirely and rely on
    // `str::eq_ignore_ascii_case` per filler against a slice of
    // the haystack. All FILLERS entries are ASCII, so case-insensitive
    // ASCII matching is sufficient and never needs lowercasing the
    // whole haystack. `haystack.get(idx..idx+f.len())` returns
    // `None` when `idx + f.len()` would split a multibyte char,
    // which makes the match harmlessly fail for non-ASCII regions.
    //
    // The byte-level boundary checks (`bytes[idx-1]` /
    // `bytes[after]`) stay safe: `is_ascii_alphanumeric` returns
    // `false` for any byte outside ASCII (including UTF-8
    // continuation bytes), so a multibyte char trivially counts as
    // a word boundary — correct: `£200um` legitimately has a word
    // break between `£` and `um`.
    let mut result = String::with_capacity(haystack.len());
    let mut last_end = 0usize;
    let mut idx = 0usize;
    let bytes = haystack.as_bytes();
    while idx < bytes.len() {
        let mut matched: Option<(usize, usize)> = None;
        for f in FILLERS {
            let after = idx + f.len();
            let Some(window) = haystack.get(idx..after) else {
                // Either out of range, or `idx`/`after` lands
                // mid-multibyte. Either way this filler can't match
                // here -- skip and try the next one.
                continue;
            };
            if window.eq_ignore_ascii_case(f) {
                let at_start = idx == 0 || !bytes[idx - 1].is_ascii_alphanumeric();
                let at_end = after >= bytes.len() || !bytes[after].is_ascii_alphanumeric();
                if at_start && at_end {
                    matched = Some((idx, after));
                    break;
                }
            }
        }
        if let Some((s, e)) = matched {
            result.push_str(&haystack[last_end..s]);
            idx = e;
            // gobble one trailing space or comma so "uh, hello" collapses to "hello"
            // rather than ", hello". `last_end` captures the position after that gobble.
            while idx < bytes.len() && matches!(bytes[idx], b' ' | b',') {
                idx += 1;
            }
            last_end = idx;
        } else {
            idx += 1;
        }
    }
    result.push_str(&haystack[last_end..]);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_fillers() {
        let cleaned = strip_fillers("Hello, um, this is a, like, test.");
        assert!(!cleaned.to_lowercase().contains(" um"));
        assert!(!cleaned.to_lowercase().contains("like"));
    }

    #[test]
    fn capitalises_sentences() {
        let out = capitalise_sentences("hello. this is a test. yes.");
        assert!(out.starts_with("Hello"));
        assert!(out.contains(". This"));
    }

    #[test]
    fn adds_terminal_punctuation() {
        assert_eq!(ensure_terminal_punctuation("hello"), "hello.");
        assert_eq!(ensure_terminal_punctuation("hello!"), "hello!");
    }

    // ── v0.15.6 UTF-8 safety regressions ──────────────────────────────
    //
    // Crashed 2026-04-29 on a real Parakeet dictation. Repro string
    // pinned verbatim. The fix's contract: never panic on multibyte
    // chars, regardless of which filler / haystack slice combination
    // happens to step into one.

    /// **The exact crash string from the v0.15.6 QA run.** Must
    /// not panic. Output need not match a specific shape — we
    /// just want the function to return.
    #[test]
    fn strip_fillers_does_not_panic_on_pound_sign_dictation() {
        let input = "wait, is the whole order £200 or are you getting £10 on i'm getting £20 or we splitting £100?";
        let out = strip_fillers(input);
        // Sanity: pound symbols and digits survive. We're not
        // pinning exact whitespace because the filler scan may
        // swallow a comma; the £ values are the canary.
        assert!(out.contains("£200"));
        assert!(out.contains("£10"));
        assert!(out.contains("£20"));
        assert!(out.contains("£100"));
    }

    /// Multibyte char immediately before a filler. The byte index
    /// before `um` is the second byte of `£`; the previous version
    /// would have indexed mid-char.
    #[test]
    fn strip_fillers_handles_multibyte_before_filler() {
        let out = strip_fillers("price is £200 um thanks");
        // No panic; `um` should be removed and trailing space gobbled.
        assert!(out.contains("£200"));
        assert!(!out.to_lowercase().contains(" um "));
    }

    /// Multibyte char immediately after a filler.
    #[test]
    fn strip_fillers_handles_multibyte_after_filler() {
        let out = strip_fillers("um £200 is fine");
        assert!(out.contains("£200"));
        assert!(!out.to_lowercase().starts_with("um "));
    }

    /// ASCII regression: existing filler-removal still works for
    /// pure-ASCII inputs (the most common case).
    #[test]
    fn strip_fillers_ascii_baseline_still_works() {
        let out = strip_fillers("Hello, um, this is a, like, test.");
        let lower = out.to_lowercase();
        // Word-bounded "um" and "like" should be gone; "this is a"
        // and "test" should remain. We don't pin exact whitespace
        // because the comma-gobble behaviour is part of the design.
        assert!(!lower.contains(" um "));
        assert!(!lower.contains("like,"));
        assert!(out.contains("Hello"));
        assert!(out.contains("test"));
    }

    /// Emoji (4-byte UTF-8) and other multibyte chars must not
    /// panic. Pin a few representative cases that span the
    /// 2/3/4-byte UTF-8 width range.
    #[test]
    fn strip_fillers_handles_assorted_multibyte_chars() {
        // 4-byte emoji
        let _ = strip_fillers("hello 🚀 um world");
        // 3-byte CJK
        let _ = strip_fillers("price 円 um thanks");
        // Combining diacritics
        let _ = strip_fillers("café um latte");
        // The filler check at every byte position even when no
        // filler matches — pin a string of pure non-ASCII.
        let _ = strip_fillers("£€¥₹");
    }

    // ── v1 Smart rewrite contract ────────────────────────────────────
    //
    // The user-visible "Smart rewrite" toggle in Settings → Text
    // Output runs `DeterministicProvider::rewrite_sync` on regular
    // (non-meeting) dictation output. Pin the canonical "um this is
    // a test" example from the v1 brief so a future change to the
    // deterministic provider can't silently change what the toggle
    // produces. Ditto for the no-LLM, no-network guarantee — the
    // toggle is "deterministic only" per its description copy.

    /// Crate-internal smoke that the smart-rewrite path called by
    /// the dictation pipeline (`DeterministicProvider.rewrite_sync`)
    /// turns the brief's exact example into clean prose. Stable
    /// across runs; no clock, no IO.
    #[test]
    fn smart_rewrite_cleans_the_canonical_example() {
        let out = DeterministicProvider.rewrite_sync("um this is a test");
        // The whitespace / filler / capitalisation transforms run in
        // a fixed order:
        //   1. normalise_whitespace
        //   2. strip_fillers ("um " → "")
        //   3. ensure_terminal_punctuation (".")
        //   4. capitalise_sentences ("t" → "T")
        // We assert on observable properties rather than an exact
        // string so a future tweak to e.g. the comma-gobble rule
        // doesn't break this test for an unrelated reason.
        let lower = out.to_lowercase();
        assert!(
            !lower.contains("um "),
            "filler 'um ' must be stripped; got: {out:?}"
        );
        assert!(
            out.starts_with('T'),
            "first letter must be capitalised; got: {out:?}"
        );
        assert!(
            out.ends_with('.'),
            "terminal punctuation must be added; got: {out:?}"
        );
        // After capitalise_sentences, "this" becomes "This"; assert
        // case-insensitively so the test pins the content not the
        // capitalisation (capitalisation is asserted on its own
        // above).
        assert!(
            out.to_lowercase().contains("this is a test"),
            "core content must survive; got: {out:?}"
        );
    }

    /// `rewrite_sync` is purely string-in/string-out. No async, no
    /// IO, no network. This test does not prove "no network"
    /// directly — there is no syscall hook in unit tests — but it
    /// pins the synchronous signature, which is the contract the
    /// dictation pipeline relies on to know smart-rewrite cannot
    /// initiate a remote call. Removing `rewrite_sync` or making
    /// it async would break this test, surface in code review, and
    /// require updating the dictation post-processing block.
    #[test]
    fn smart_rewrite_is_sync_and_deterministic() {
        // Run twice on the same input; output must be byte-identical.
        let input = "um, this is a, like, test of the rewrite path";
        let a = DeterministicProvider.rewrite_sync(input);
        let b = DeterministicProvider.rewrite_sync(input);
        assert_eq!(
            a, b,
            "deterministic rewrite must be byte-stable across repeated calls"
        );
    }

    /// Empty / whitespace-only inputs must not panic and must not
    /// produce surprising characters. The dictation pipeline can
    /// route empty strings through here (e.g. silence-trimmed
    /// recordings) and we don't want a stray full stop showing up.
    #[test]
    fn smart_rewrite_handles_empty_input() {
        assert_eq!(DeterministicProvider.rewrite_sync(""), "");
        assert_eq!(DeterministicProvider.rewrite_sync("   "), "");
    }

    /// Edge-input contract for Smart rewrite v1.
    ///
    /// The deterministic rewrite is rule-based: it adds a terminal
    /// full stop, capitalises sentence starts, removes a small list
    /// of fillers, and normalises whitespace. It does NOT detect
    /// code, abbreviations, or context. Users who opt the toggle
    /// in are accepting these limitations explicitly via the copy:
    /// "Deterministic only — no AI model or cloud service. Adds a
    /// small amount of processing time. Off by default."
    ///
    /// This test captures the observed v1 behaviour on every edge
    /// input the review brief called out, so a future tweak to any
    /// of the building-block helpers (strip_fillers, capitalise_
    /// sentences, ensure_terminal_punctuation) can't silently
    /// change what the toggle produces. Each row is what shipped;
    /// every row is also "acceptable for an opt-in beta toggle".
    /// If a row needs to change, the toggle's copy needs to change
    /// in lockstep.
    #[test]
    fn smart_rewrite_edge_input_contract() {
        // (input, expected_output) pairs.
        let cases = [
            // Short commands: capitalised + terminal full stop.
            // Fine for a chat-style "yes/no/send/stop" dictation —
            // most apps accept the trailing dot, and the
            // capitalisation matches sentence-style output the
            // user opted into.
            ("send", "Send."),
            ("stop", "Stop."),
            ("yes", "Yes."),
            ("no", "No."),
            ("Cancel", "Cancel."),
            // Code snippets are explicitly out of scope. The
            // rewrite WILL capitalise the first word ("let" ->
            // "Let") and may capitalise after `;` (next-sentence
            // marker). Users dictating code should leave the
            // toggle off; the copy says "Clean up fillers,
            // spacing, capitalisation, and obvious punctuation"
            // — not "smart enough for code".
            ("let foo = bar()", "Let foo = bar()."),
            // Acronyms in caps are preserved (no rule lowercases
            // alpha runs).
            ("API", "API."),
            ("GDPR", "GDPR."),
            // Abbreviations with internal dots: each `.` flips the
            // capitalise-next flag, so "i.e." becomes "I.E.".
            // Conventionally these are lowercase. v1 limitation;
            // acceptable for opt-in.
            ("i.e.", "I.E."),
            ("e.g.", "E.G."),
            // Common "X." patterns where the input was already
            // sentence-shaped fare better.
            ("Mr. Smith", "Mr. Smith."),
            ("Dr.", "Dr."),
        ];
        for (input, expected) in cases {
            let out = DeterministicProvider.rewrite_sync(input);
            assert_eq!(
                out, expected,
                "v1 contract broke on input {input:?}: got {out:?}, expected {expected:?}. \
                 If this is a deliberate change, also update the toggle's description \
                 copy in TextOutputSettings.tsx to match the new behaviour."
            );
        }
    }
}
