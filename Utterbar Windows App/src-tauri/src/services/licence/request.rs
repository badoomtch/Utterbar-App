//! Activation-request file builder for air-gapped activation.
//!
//! An offline machine can't talk to `/api/licence/activate`, so the IT
//! admin needs the machine's identity (fingerprint, name, app version)
//! out-of-band. This module builds the JSON document the user saves to
//! disk and hand-carries / emails to their admin. The admin uploads it
//! at `/admin/licences/{id}` (see `LicenceController::uploadActivationRequest`
//! on the website), which records an `activations` row with
//! `offline = true` and enforces `max_devices` server-side.
//!
//! ## Format
//!
//! ```json
//! {
//!   "format":         "utterbar-licence-request",
//!   "format_version": 1,
//!   "fingerprint":    "<64 lowercase hex chars>",
//!   "device_name":    "<COMPUTERNAME or hostname>",
//!   "platform":       "windows",
//!   "app_version":    "0.15.11",
//!   "generated_at":   "2026-05-03T14:25:01Z"
//! }
//! ```
//!
//! Why JSON: the admin wants to eyeball the file before uploading, the
//! file is small enough that human readability beats compactness, and
//! the website's request-upload validator can lean on Laravel's JSON
//! decoder rather than parsing a custom binary frame.
//!
//! Why not signed: nothing in the file is a secret. The fingerprint
//! the request leaks is the same fingerprint the online `/activate`
//! endpoint already trusts. Signing it would require a second keypair
//! the desktop doesn't currently have, with no security gain — the
//! fingerprint is bound to the machine that ran `compute_fingerprint`,
//! and the admin's upload-validation step is the trust gate.
//!
//! ## What the request does NOT do
//!
//! It does not bind the eventual `licence.bin` to this machine. The
//! signed blob the admin returns is the same blob every other holder
//! of this licence row gets. A v3 payload that embeds the fingerprint
//! into the signed bytes is on the parking lot; see `services/licence/
//! verify.rs` for the v2 reserved-bytes layout that would carry it,
//! and `docs/it-deployment` for the limitation as documented to
//! customers.

use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::util::result::OrliqError;

/// Magic string identifying this as an Utterbar request file. The
/// website's upload validator rejects anything that doesn't carry this
/// exact string; bumping it (or `format_version` below) is a forward-
/// compatibility signal.
pub const FORMAT_MAGIC: &str = "utterbar-licence-request";

/// Schema version. Bump when adding fields the website MUST consume.
/// The website's upload validator currently accepts only `1`; a future
/// `2` carrying additional metadata can be wired in lockstep.
pub const FORMAT_VERSION: u32 = 1;

/// Length of the fingerprint prefix used in the suggested filename.
/// 12 hex chars = 48 bits, plenty for a human-readable disambiguator
/// when an admin has multiple request files in their inbox.
pub const FILENAME_FP_PREFIX_LEN: usize = 12;

/// Wire shape of the request file. Field order is presentation-only —
/// the JSON parser on the admin side reads keys, not positions — but
/// matches the documented format above.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActivationRequest {
    pub format: String,
    pub format_version: u32,
    /// 64-char lowercase hex from `compute_fingerprint`.
    pub fingerprint: String,
    pub device_name: String,
    /// Hard-coded "windows" today. Reserved for the day a port lands.
    pub platform: String,
    /// `CARGO_PKG_VERSION` at compile time of the running build.
    pub app_version: String,
    /// RFC 3339 UTC. Audit hint — admin sees roughly when the
    /// request was generated, useful for spotting stale files in
    /// the upload queue.
    pub generated_at: String,
}

impl ActivationRequest {
    /// Build a request from raw inputs. Pure function; the caller owns
    /// fingerprint resolution + the wall clock so this stays trivial
    /// to test.
    pub fn build(
        fingerprint: String,
        device_name: String,
        app_version: String,
        generated_at: String,
    ) -> Self {
        Self {
            format: FORMAT_MAGIC.to_string(),
            format_version: FORMAT_VERSION,
            fingerprint,
            device_name,
            platform: "windows".to_string(),
            app_version,
            generated_at,
        }
    }

    /// Build a request for the current machine. Reads the
    /// fingerprint + device name from
    /// `platform::windows::fingerprint`, the app version from
    /// `CARGO_PKG_VERSION`, and the timestamp from the system clock
    /// formatted as RFC 3339 UTC.
    ///
    /// On non-Windows the function is a compile error today (no
    /// cross-platform fingerprint impl exists). The Tauri command
    /// gates this call behind the same `#[cfg(windows)]` the rest
    /// of the licence stack uses.
    #[cfg(windows)]
    pub fn for_current_machine() -> Result<Self, OrliqError> {
        let fingerprint =
            crate::platform::windows::fingerprint::compute_fingerprint().map_err(|e| {
                OrliqError::Licence {
                    code: "internal".to_string(),
                    message: format!("fingerprint compute: {e}"),
                }
            })?;
        let device_name = crate::platform::windows::fingerprint::device_name();
        let now = OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .map_err(|e| OrliqError::Licence {
                code: "internal".to_string(),
                message: format!("timestamp format: {e}"),
            })?;
        Ok(Self::build(
            fingerprint,
            device_name,
            env!("CARGO_PKG_VERSION").to_string(),
            now,
        ))
    }

    /// Suggested filename for `saveDialog`. Carries the first 12
    /// hex chars of the fingerprint so an admin staring at a folder
    /// of requests can tell them apart without opening each one.
    pub fn suggested_filename(&self) -> String {
        let prefix: String = self
            .fingerprint
            .chars()
            .take(FILENAME_FP_PREFIX_LEN)
            .collect();
        format!("utterbar-request-{prefix}.json")
    }

    /// Pretty-printed JSON. The admin will read this with their
    /// eyes; compact JSON would save bytes nobody cares about.
    pub fn to_json_pretty(&self) -> Result<String, OrliqError> {
        serde_json::to_string_pretty(self).map_err(|e| OrliqError::Licence {
            code: "internal".to_string(),
            message: format!("serialise request: {e}"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> ActivationRequest {
        ActivationRequest::build(
            "a".repeat(64),
            "TEST-PC".to_string(),
            "0.15.11".to_string(),
            "2026-05-03T14:25:01Z".to_string(),
        )
    }

    /// The wire shape MUST round-trip through JSON unchanged so the
    /// website upload validator and the desktop builder agree on
    /// every field. A regression here would silently break the admin
    /// upload path.
    #[test]
    fn request_round_trips_through_json() {
        let req = sample();
        let json = req.to_json_pretty().expect("serialise");
        let back: ActivationRequest = serde_json::from_str(&json).expect("parse");
        assert_eq!(req, back);
    }

    /// Pin the format magic + version. The website validator hard-
    /// codes `"utterbar-licence-request"` and `1`; if either drifts,
    /// the admin upload would 422 with no obvious cause.
    #[test]
    fn format_magic_and_version_are_pinned() {
        let req = sample();
        assert_eq!(req.format, "utterbar-licence-request");
        assert_eq!(req.format_version, 1);
    }

    /// Compile-time constants are the real source of truth — a
    /// future bump of either should require deliberate edits, not
    /// a near-silent number change.
    #[test]
    fn format_constants_are_stable() {
        assert_eq!(FORMAT_MAGIC, "utterbar-licence-request");
        assert_eq!(FORMAT_VERSION, 1);
    }

    /// Filename prefix length is documented at 12 chars — checked
    /// here so a future "let's bump it to 16" change doesn't slip
    /// past code review without updating the docstring + the docs
    /// page.
    #[test]
    fn suggested_filename_uses_12_hex_chars() {
        let req = sample();
        let name = req.suggested_filename();
        assert!(
            name.starts_with("utterbar-request-"),
            "filename must start with the documented prefix: {name}"
        );
        assert!(name.ends_with(".json"), "filename must end .json: {name}");
        // "utterbar-request-" (17) + 12 fp chars + ".json" (5) = 34.
        assert_eq!(name.len(), 34, "filename length: {name}");
        // Pin the exact slice — guards against an off-by-one in the
        // prefix slice landing somewhere odd in the fingerprint.
        assert_eq!(name, "utterbar-request-aaaaaaaaaaaa.json");
    }

    /// Field order in the JSON output is presentation-only, but a
    /// human reading the file expects `format` to be the first key.
    /// Pin that so a future field shuffle doesn't silently reorder
    /// the document an admin is going to open in Notepad.
    #[test]
    fn json_has_format_field_first() {
        let req = sample();
        let json = req.to_json_pretty().expect("serialise");
        let format_idx = json.find("\"format\"").expect("format key present");
        let fingerprint_idx = json
            .find("\"fingerprint\"")
            .expect("fingerprint key present");
        assert!(
            format_idx < fingerprint_idx,
            "format key should appear before fingerprint key:\n{json}"
        );
    }

    /// The platform is hard-coded today. Fixing this in a test means
    /// the day a Mac/Linux port lands the test guides the change to
    /// the right place rather than letting a stale "windows" string
    /// ship in a request from a non-Windows build.
    #[test]
    fn platform_is_windows() {
        let req = sample();
        assert_eq!(req.platform, "windows");
    }
}
