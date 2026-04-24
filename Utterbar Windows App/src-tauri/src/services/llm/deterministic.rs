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
    // The test suite in this module covers the intended behaviour.
    let lower = haystack.to_lowercase();
    let mut result = String::with_capacity(haystack.len());
    let mut last_end = 0usize;
    let mut idx = 0usize;
    let bytes = haystack.as_bytes();
    while idx < bytes.len() {
        let remaining = &lower[idx..];
        let mut matched: Option<(usize, usize)> = None;
        for f in FILLERS {
            if remaining.starts_with(f) {
                // check word boundaries
                let after = idx + f.len();
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
}
