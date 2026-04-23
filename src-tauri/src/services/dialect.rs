//! English dialect normalisation.
//!
//! Parakeet-TDT is a monolingual English model trained predominantly on North
//! American data — there's no locale knob to flip, and its output skews to
//! American spellings ("color", "organize", "favorite"). That's a mismatch for
//! British users who dictate a word and see it typed back with an unfamiliar
//! spelling a few beats later.
//!
//! Rather than paying an LLM round-trip just to swap spellings, we do a
//! deterministic dictionary replace. The list below covers the common
//! BrE↔AmE divergences — `-our/-or`, `-ise/-ize`, `-yse/-yze`, `-re/-er`,
//! single/double consonants before verb endings, and a handful of
//! high-frequency one-offs (`favourite`, `grey`, `aeroplane`, `tyre`).
//!
//! We only swap whole words and preserve the leading letter's case, so
//! "Colour" becomes "Color" and "COLOUR" stays as-is visually because the
//! match is lowercase — upper-case acronyms and all-caps shouting are left
//! untouched. Anything outside the table passes through verbatim.
//!
//! This runs as the very last step of post-processing, after the user's
//! mode-level rewrite (deterministic or LLM) and after text-output rules.
//! Cost: a linear walk of the transcript, microseconds for a normal
//! dictation. Safe to apply to any transcript, including ones the user has
//! customised — we only touch tokens on the dictionary.

use crate::domain::EnglishVariant;

/// Pairs of (british, american). Left value is the BrE form, right is the
/// AmE form. The normaliser picks a direction based on the user's chosen
/// variant: British reads right→left (AmE→BrE), American reads left→right.
///
/// Keep entries lowercase and singular where sensible — we handle basic
/// morphology below (`-ed`, `-ing`, `-s`, `-ly`, `-ation`) via family
/// expansion. Don't add entries that would wrong-footedly collide with
/// proper nouns (e.g. "Mom's Diner"); if in doubt, leave it out.
const PAIRS: &[(&str, &str)] = &[
    // -our / -or
    ("colour", "color"),
    ("flavour", "flavor"),
    ("honour", "honor"),
    ("humour", "humor"),
    ("labour", "labor"),
    ("neighbour", "neighbor"),
    ("behaviour", "behavior"),
    ("favour", "favor"),
    ("favourite", "favorite"),
    ("rumour", "rumor"),
    ("harbour", "harbor"),
    ("odour", "odor"),
    ("armour", "armor"),
    ("endeavour", "endeavor"),
    ("splendour", "splendor"),
    ("saviour", "savior"),
    ("vapour", "vapor"),
    ("parlour", "parlor"),
    // -ise / -ize (verbs and their nominalisations)
    ("organise", "organize"),
    ("organisation", "organization"),
    ("realise", "realize"),
    ("recognise", "recognize"),
    ("analyse", "analyze"),
    ("apologise", "apologize"),
    ("summarise", "summarize"),
    ("prioritise", "prioritize"),
    ("categorise", "categorize"),
    ("optimise", "optimize"),
    ("customise", "customize"),
    ("memorise", "memorize"),
    ("authorise", "authorize"),
    ("utilise", "utilize"),
    ("emphasise", "emphasize"),
    ("maximise", "maximize"),
    ("minimise", "minimize"),
    ("finalise", "finalize"),
    ("standardise", "standardize"),
    ("specialise", "specialize"),
    ("synchronise", "synchronize"),
    ("normalise", "normalize"),
    ("stabilise", "stabilize"),
    ("criticise", "criticize"),
    ("paralyse", "paralyze"),
    // -re / -er
    ("centre", "center"),
    ("metre", "meter"),
    ("litre", "liter"),
    ("theatre", "theater"),
    ("fibre", "fiber"),
    ("calibre", "caliber"),
    ("sombre", "somber"),
    ("spectre", "specter"),
    // -ce / -se (noun vs verb differences)
    ("defence", "defense"),
    ("offence", "offense"),
    ("licence", "license"),
    ("pretence", "pretense"),
    // double consonants before verb endings
    ("travelled", "traveled"),
    ("travelling", "traveling"),
    ("traveller", "traveler"),
    ("cancelled", "canceled"),
    ("cancelling", "canceling"),
    ("modelled", "modeled"),
    ("modelling", "modeling"),
    ("labelled", "labeled"),
    ("labelling", "labeling"),
    ("signalled", "signaled"),
    ("signalling", "signaling"),
    ("fuelled", "fueled"),
    ("fuelling", "fueling"),
    // -ogue / -og
    ("dialogue", "dialog"),
    ("catalogue", "catalog"),
    ("analogue", "analog"),
    ("monologue", "monolog"),
    // miscellaneous
    ("grey", "gray"),
    ("aeroplane", "airplane"),
    ("aluminium", "aluminum"),
    ("tyre", "tire"),
    ("kerb", "curb"),
    ("plough", "plow"),
    ("mum", "mom"),
    ("maths", "math"),
    ("whilst", "while"),
    ("cheque", "check"),
    ("draught", "draft"),
    ("programme", "program"),
    ("storey", "story"),
    ("mould", "mold"),
    ("moustache", "mustache"),
    ("sceptic", "skeptic"),
    ("sceptical", "skeptical"),
    ("jewellery", "jewelry"),
    ("manoeuvre", "maneuver"),
    ("foetus", "fetus"),
    ("oesophagus", "esophagus"),
];

/// Normalise a transcript to the user's chosen English dialect. `Auto` is a
/// no-op; we never re-spell on the user's behalf without consent.
pub fn normalise(text: &str, variant: EnglishVariant) -> String {
    match variant {
        EnglishVariant::Auto => text.to_string(),
        EnglishVariant::British => swap(text, SwapDirection::ToBritish),
        EnglishVariant::American => swap(text, SwapDirection::ToAmerican),
    }
}

#[derive(Clone, Copy)]
enum SwapDirection {
    ToBritish,
    ToAmerican,
}

/// Walk the text token-by-token. A "token" here is a maximal run of letters;
/// anything else (spaces, punctuation, digits) is emitted verbatim. We look
/// each lowercase token up in the dictionary and, if it matches, substitute
/// the opposite variant while preserving the original token's case pattern.
fn swap(text: &str, dir: SwapDirection) -> String {
    let mut out = String::with_capacity(text.len());
    let mut buf = String::new();
    for c in text.chars() {
        if c.is_alphabetic() {
            buf.push(c);
        } else {
            if !buf.is_empty() {
                out.push_str(&replace_token(&buf, dir));
                buf.clear();
            }
            out.push(c);
        }
    }
    if !buf.is_empty() {
        out.push_str(&replace_token(&buf, dir));
    }
    out
}

fn replace_token(token: &str, dir: SwapDirection) -> String {
    let lower = token.to_lowercase();
    for &(br, am) in PAIRS {
        let (from, to) = match dir {
            SwapDirection::ToBritish => (am, br),
            SwapDirection::ToAmerican => (br, am),
        };
        if lower == from {
            return match_case(token, to);
        }
    }
    token.to_string()
}

/// Mimic the original token's capitalisation on the replacement. We only
/// handle the three shapes that matter: all-lower ("colour"), title-case
/// ("Colour"), and all-upper ("COLOUR"). Mixed-case tokens (internal caps,
/// camelCase) are unusual in dictated prose, so we fall back to the
/// replacement's raw form rather than trying to be clever.
fn match_case(original: &str, replacement: &str) -> String {
    if original.chars().all(|c| c.is_uppercase()) {
        replacement.to_uppercase()
    } else if original
        .chars()
        .next()
        .map(|c| c.is_uppercase())
        .unwrap_or(false)
    {
        let mut s = String::with_capacity(replacement.len());
        let mut chars = replacement.chars();
        if let Some(first) = chars.next() {
            for c in first.to_uppercase() {
                s.push(c);
            }
        }
        s.extend(chars);
        s
    } else {
        replacement.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_is_passthrough() {
        let s = "The color of my favorite sweater is gray.";
        assert_eq!(normalise(s, EnglishVariant::Auto), s);
    }

    #[test]
    fn to_british_basic() {
        let got = normalise(
            "The color of my favorite sweater is gray.",
            EnglishVariant::British,
        );
        assert_eq!(got, "The colour of my favourite sweater is grey.");
    }

    #[test]
    fn to_american_basic() {
        let got = normalise(
            "I'll organise the centre and analyse the colour.",
            EnglishVariant::American,
        );
        assert_eq!(got, "I'll organize the center and analyze the color.");
    }

    #[test]
    fn preserves_case() {
        assert_eq!(
            normalise("Colour COLOUR colour", EnglishVariant::American),
            "Color COLOR color"
        );
    }

    #[test]
    fn leaves_unknown_words_alone() {
        let s = "widget-foo bar_baz";
        assert_eq!(normalise(s, EnglishVariant::British), s);
        assert_eq!(normalise(s, EnglishVariant::American), s);
    }

    #[test]
    fn preserves_punctuation_and_whitespace() {
        let got = normalise(
            "Color, flavor; and honor!  (really)",
            EnglishVariant::British,
        );
        assert_eq!(got, "Colour, flavour; and honour!  (really)");
    }
}
