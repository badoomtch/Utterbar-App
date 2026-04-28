//! Meeting-mode commands. Currently scoped to the speaker-rename
//! surface introduced in v0.15.0 alongside the diarised pipeline.
//!
//! The diariser produces transcripts with `**Speaker N:**` literal
//! tokens (zero-padded to 1-based ids: Speaker 1, Speaker 2, …).
//! The user can rename those labels post-transcribe from the
//! Meetings page; the rename map persists per-row in
//! `history.speaker_names` (a JSON object). Render-time
//! substitution swaps `**Speaker N:**` for `**<renamed>:**` —
//! never mutates `processed_text`, so re-rename and clear-rename
//! always work.
//!
//! Validation rules (enforced server-side):
//!
//!   * speaker_id key: integer ≥1, ≤32. The hard cap of 32 is a
//!     defence against malformed input — sherpa would never produce
//!     more in any sensible meeting, and a higher value is almost
//!     certainly a malformed JSON object that we should reject.
//!   * name value: trimmed of leading/trailing whitespace, must be
//!     non-empty after trim, ≤64 characters, no control characters
//!     (\x00-\x1f, \x7f). Control chars would either break markdown
//!     rendering or smuggle hidden content through exports.
//!
//! Failure modes:
//!   * `set_meeting_speaker_names` rejects the entire map on the
//!     first invalid entry with a clear `OrliqError::User` message
//!     pointing at the offending key/value. We don't silently
//!     ignore bad entries — a partial rename would be confusing.
//!   * `clear_meeting_speaker_names` is unconditional: writes NULL
//!     to the column, restoring `**Speaker N:**` rendering.

use std::collections::HashMap;
use std::sync::Arc;

use tauri::State;

use crate::app_state::AppState;
use crate::util::result::{CmdResult, OrliqError};

/// Hard upper bound on speaker ids. sherpa's threshold-clustering
/// in a sensible meeting setting produces single-digit cluster
/// counts; if the rename map carries an id higher than this it's
/// almost certainly a malformed JSON object that we should refuse.
const MAX_SPEAKER_ID: i64 = 32;

/// Hard upper bound on rename name length. Matches the existing
/// mode display_name cap, so the rename surface and the
/// existing-mode picker share the same UX bound.
const MAX_NAME_LEN: usize = 64;

/// Render-time validation result on a single (id, name) entry.
fn validate_entry(id_str: &str, name: &str) -> Result<(i64, String), OrliqError> {
    let id: i64 = id_str.parse().map_err(|_| OrliqError::User {
        message: format!("speaker rename: id key {id_str:?} is not a valid integer"),
    })?;
    if !(1..=MAX_SPEAKER_ID).contains(&id) {
        return Err(OrliqError::User {
            message: format!(
                "speaker rename: id {id} is out of range (must be 1..={MAX_SPEAKER_ID})"
            ),
        });
    }
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(OrliqError::User {
            message: format!(
                "speaker rename: name for id {id} is empty after trim — use clear_meeting_speaker_names to revert to defaults"
            ),
        });
    }
    if trimmed.chars().count() > MAX_NAME_LEN {
        return Err(OrliqError::User {
            message: format!(
                "speaker rename: name for id {id} is too long (max {MAX_NAME_LEN} characters)"
            ),
        });
    }
    if trimmed
        .chars()
        .any(|c| (c as u32) < 0x20 || c as u32 == 0x7f)
    {
        return Err(OrliqError::User {
            message: format!(
                "speaker rename: name for id {id} contains control characters; only printable text is allowed"
            ),
        });
    }
    Ok((id, trimmed.to_string()))
}

/// Set the per-meeting speaker rename map. Validates every entry
/// against the rules above; rejects the entire map on the first
/// invalid entry. Persists the map as a serialised JSON string
/// keyed by integer-string speaker id.
#[tauri::command]
pub async fn set_meeting_speaker_names(
    state: State<'_, Arc<AppState>>,
    history_id: String,
    names: HashMap<String, String>,
) -> CmdResult<()> {
    let mut validated: HashMap<String, String> = HashMap::with_capacity(names.len());
    for (id_str, name) in &names {
        let (id, trimmed) = validate_entry(id_str, name)?;
        // Re-key on the canonicalised integer-as-string so the
        // wire shape is always `"1"`, `"2"`, etc. — never `"01"`
        // or `"+1"`. Frontend regex matches against the canonical
        // form.
        validated.insert(id.to_string(), trimmed);
    }
    let json = serde_json::to_string(&validated).map_err(|e| OrliqError::User {
        message: format!("failed to serialise rename map: {e}"),
    })?;
    state
        .history
        .update_speaker_names(&history_id, Some(&json))
        .map_err(map_history_update_err)?;
    Ok(())
}

/// Clear the per-meeting speaker rename map. After this, the row
/// renders the raw `**Speaker N:**` labels from `processed_text`
/// again. Errors when the row doesn't exist or isn't a meeting —
/// the frontend rolls back its optimistic state on that path.
#[tauri::command]
pub async fn clear_meeting_speaker_names(
    state: State<'_, Arc<AppState>>,
    history_id: String,
) -> CmdResult<()> {
    state
        .history
        .update_speaker_names(&history_id, None)
        .map_err(map_history_update_err)?;
    Ok(())
}

/// Map a `HistoryService::update_speaker_names` error into a
/// user-facing `OrliqError::User` rather than a generic Internal
/// one. The "not found" message comes from the service layer when
/// the UPDATE affected zero rows (missing id or non-meeting row);
/// surfacing it as User means the frontend's `alert(e.message)` and
/// rollback path render a clean message rather than a stack trace.
fn map_history_update_err(err: anyhow::Error) -> OrliqError {
    let msg = err.to_string();
    if msg.contains("not found") {
        OrliqError::User {
            message: "Meeting transcript not found.".into(),
        }
    } else {
        OrliqError::Internal { message: msg }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_name_after_trim() {
        let r = validate_entry("1", "   ");
        assert!(r.is_err());
        let r2 = validate_entry("1", "");
        assert!(r2.is_err());
    }

    #[test]
    fn rejects_overlong_name() {
        let long = "A".repeat(MAX_NAME_LEN + 1);
        assert!(validate_entry("1", &long).is_err());
    }

    #[test]
    fn rejects_control_chars() {
        // Control chars in the middle of the name (where they
        // can't be trimmed away) MUST be rejected — they'd break
        // markdown rendering or smuggle hidden content into
        // exports. Control chars at the edges are handled by the
        // initial trim() pass; the test below pins the worse case.
        for c in ['\x00', '\x01', '\x07', '\x1f', '\x7f', '\n', '\t', '\r'] {
            let s = format!("Ja{c}y");
            let r = validate_entry("1", &s);
            assert!(r.is_err(), "expected reject for {c:?} mid-name, got {r:?}");
        }
    }

    #[test]
    fn trims_edge_whitespace_and_newlines() {
        // Trailing whitespace / newline is silently trimmed. This
        // is the documented behaviour: the trim() pass runs before
        // the control-char check.
        let (_, name) = validate_entry("1", "Jay\n").unwrap();
        assert_eq!(name, "Jay");
        let (_, name) = validate_entry("1", "  Jay  \t").unwrap();
        assert_eq!(name, "Jay");
    }

    #[test]
    fn rejects_speaker_id_above_cap() {
        assert!(validate_entry("33", "Jay").is_err());
        assert!(validate_entry("99", "Jay").is_err());
    }

    #[test]
    fn rejects_speaker_id_below_one() {
        assert!(validate_entry("0", "Jay").is_err());
        assert!(validate_entry("-1", "Jay").is_err());
    }

    #[test]
    fn rejects_non_integer_id() {
        assert!(validate_entry("one", "Jay").is_err());
        assert!(validate_entry("1.5", "Jay").is_err());
    }

    #[test]
    fn accepts_valid_entries_and_trims_whitespace() {
        let (id, name) = validate_entry("1", "  Jay  ").unwrap();
        assert_eq!(id, 1);
        assert_eq!(name, "Jay");
    }

    #[test]
    fn accepts_unicode_names() {
        // Non-ASCII letters are fine — only control chars are rejected.
        let (_, name) = validate_entry("2", "Jürgen").unwrap();
        assert_eq!(name, "Jürgen");
        let (_, name) = validate_entry("3", "李明").unwrap();
        assert_eq!(name, "李明");
    }

    /// `map_history_update_err` recognises the "not found" string
    /// from the service layer and surfaces it as `OrliqError::User`,
    /// which the frontend can render as a friendly alert. Other
    /// errors stay as `Internal` so they don't masquerade as a
    /// user-correctable issue.
    #[test]
    fn maps_not_found_to_user_error() {
        let err = anyhow::anyhow!("meeting transcript not found");
        match map_history_update_err(err) {
            OrliqError::User { message } => {
                assert!(message.contains("Meeting transcript not found"));
            }
            other => panic!("expected User error, got {other:?}"),
        }
    }

    #[test]
    fn maps_other_errors_to_internal() {
        let err = anyhow::anyhow!("database is locked");
        match map_history_update_err(err) {
            OrliqError::Internal { message } => {
                assert!(message.contains("database is locked"));
            }
            other => panic!("expected Internal error, got {other:?}"),
        }
    }
}
