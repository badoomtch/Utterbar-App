//! Licence key normalisation + validation.
//!
//! Six rules from docs/LICENCE_APP_HANDOFF §2, runnable client-side to avoid
//! a server round-trip on obvious typos. Canonical form is uppercase,
//! dash-free, 20 chars, Crockford base32 alphabet (0-9 + A-Z minus I, L, O, U).

use crate::util::result::OrliqError;

const CANONICAL_LEN: usize = 20;

/// Apply the six normalisation rules and return the canonical 20-char form.
/// Returns `OrliqError::Licence { kind: "malformed", ... }` on any violation —
/// the frontend presents this inline on the paste field.
pub fn normalise(input: &str) -> Result<String, OrliqError> {
    let stripped: String = input
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .flat_map(char::to_uppercase)
        .collect();

    // Drop the human-friendly `UB` prefix if present.
    let trimmed = stripped.strip_prefix("UB").unwrap_or(&stripped);

    if trimmed.len() != CANONICAL_LEN {
        return Err(OrliqError::Licence {
            code: "malformed".to_string(),
            message: format!(
                "licence key must be {CANONICAL_LEN} chars after stripping dashes and the UB prefix; got {}",
                trimmed.len()
            ),
        });
    }

    if !trimmed.chars().all(is_crockford) {
        return Err(OrliqError::Licence {
            code: "malformed".to_string(),
            message:
                "licence key contains an invalid character — use digits and A-Z (except I, L, O, U)"
                    .to_string(),
        });
    }

    Ok(trimmed.to_string())
}

/// Format a canonical 20-char key back into the display form
/// `UB-XXXX-XXXX-XXXX-XXXX-XXXX`. Useful when we have the canonical form from
/// a deep-link path segment and want to show the email-style key in the UI.
pub fn format_display(canonical: &str) -> String {
    let mut out = String::with_capacity(23);
    out.push_str("UB-");
    for (i, c) in canonical.chars().enumerate() {
        if i > 0 && i % 4 == 0 {
            out.push('-');
        }
        out.push(c);
    }
    out
}

fn is_crockford(c: char) -> bool {
    // Handoff §2 rule 6: digits 0-9 and A-Z except I, L, O, U.
    c.is_ascii_digit() || (c.is_ascii_uppercase() && !matches!(c, 'I' | 'L' | 'O' | 'U'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_display_form() {
        let c = normalise("UB-7QX4-K2YM-R8FP-3JZN-WA6T").unwrap();
        assert_eq!(c, "7QX4K2YMR8FP3JZNWA6T");
    }

    #[test]
    fn accepts_already_canonical() {
        let c = normalise("7QX4K2YMR8FP3JZNWA6T").unwrap();
        assert_eq!(c, "7QX4K2YMR8FP3JZNWA6T");
    }

    #[test]
    fn accepts_lowercase_with_whitespace() {
        let c = normalise("  ub-7qx4-k2ym-r8fp-3jzn-wa6t  ").unwrap();
        assert_eq!(c, "7QX4K2YMR8FP3JZNWA6T");
    }

    #[test]
    fn rejects_wrong_length() {
        let err = normalise("UB-7QX4").unwrap_err();
        match err {
            OrliqError::Licence { code, .. } => assert_eq!(code, "malformed"),
            _ => panic!(),
        }
    }

    #[test]
    fn rejects_crockford_excluded_letters() {
        // I, L, O, U are excluded.
        for bad in ["I", "L", "O", "U"] {
            let key = format!("UB-{bad}QX4-K2YM-R8FP-3JZN-WA6T");
            let err = normalise(&key).unwrap_err();
            match err {
                OrliqError::Licence { code, message } => {
                    assert_eq!(code, "malformed");
                    assert!(message.contains("invalid character"), "got: {message}");
                }
                _ => panic!(),
            }
        }
    }

    #[test]
    fn format_display_round_trips() {
        let canonical = "7QX4K2YMR8FP3JZNWA6T";
        let display = format_display(canonical);
        assert_eq!(display, "UB-7QX4-K2YM-R8FP-3JZN-WA6T");
        // And normalising the display form returns the canonical form.
        assert_eq!(normalise(&display).unwrap(), canonical);
    }
}
