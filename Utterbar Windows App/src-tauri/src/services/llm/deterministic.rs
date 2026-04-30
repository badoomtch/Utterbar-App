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
}
