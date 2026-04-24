//! Pro-tier gating helpers.
//!
//! One place for every cap / gate, so the rules are discoverable and the
//! frontend's copy stays in sync with the backend enforcement. The tier is
//! read via `AppState::licence.status().await` at the call site — this
//! module is pure constants + small pure helpers, no state of its own.
//!
//! Caps match the public pricing copy at utterbar.com/pricing:
//!   • 50 vocabulary words
//!   • 25 replacements
//!   • 100 history entries visible
//!   • Custom modes: 0 (free can use all built-ins, not author new)
//!   • fp16 Parakeet model: Pro only
//!   • File transcription: Pro only
//!   • Audio retention (`retain_audio=true`): Pro only
//!   • Re-transcribe (`reprocess_history`): Pro only
//!
//! Over-cap entries authored on Pro are grandfathered on deactivation
//! (they remain usable and visible; "Add" and write-mutations are the
//! only things that reject). Enforcement lives inside each command module
//! close to the DB write it's guarding, not here.

use crate::domain::licence::LicenceStatus;
use crate::util::result::OrliqError;

/// Free-tier cap on user-authored vocabulary words.
pub const FREE_VOCAB_WORDS_CAP: usize = 50;
/// Free-tier cap on user-authored replacements.
pub const FREE_REPLACEMENTS_CAP: usize = 25;
/// Free-tier cap on history entries returned by `list_history`. The DB still
/// holds everything — we filter on read so upgrading to Pro retroactively
/// restores the full list without a restore step.
pub const FREE_HISTORY_CAP: usize = 100;

/// Classification flag on `ModelInfo`. fp16 (and future LLM GGUFs) carry
/// `requires_pro: true`; everything else is free. Checked inside
/// `ModelRegistry::install`.
pub fn is_pro(status: &LicenceStatus) -> bool {
    matches!(status, LicenceStatus::Pro { .. })
}

/// Convenience: build the standard "this needs Pro" user error surfaced to
/// the UI. The frontend's `ProNudge` catches this code on the generic catch
/// path and already displays the right copy, so the message text here is
/// mainly for log forensics — users see the ProNudge, not this string.
pub fn pro_required(what: &str) -> OrliqError {
    OrliqError::Licence {
        code: "pro_required".into(),
        message: format!("{what} is a Pro feature."),
    }
}

/// Convenience: "you're at the cap, upgrade for more". Same `pro_required`
/// slug so the UI treats it identically — the copy on the nudge explains
/// which cap was hit, not the error string.
pub fn cap_reached(what: &str, cap: usize) -> OrliqError {
    OrliqError::Licence {
        code: "pro_required".into(),
        message: format!("{what} cap reached ({cap}). Upgrade to Pro for unlimited."),
    }
}
