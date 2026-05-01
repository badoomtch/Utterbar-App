//! Managed (fleet) licence loader.
//!
//! IT admins can deploy a signed licence file at
//! `%APPDATA%\Utterbar\licence.bin` to license a machine without the user
//! pasting a key. The file is the same 96-byte Ed25519-signed blob the
//! website serves to admins via /admin/licences/{id}/download. Verification
//! reuses `verify::verify_signed_blob`; there is no second trust model.
//!
//! Precedence rules (per LICENCE_APP_HANDOFF + the B2B spec):
//!   - Managed licence wins if present and verifies.
//!   - Managed licence present but invalid: NEVER silently grant Pro. The
//!     loader returns an error; the orchestrator clears the in-memory
//!     status to Free and logs a warning. The cached / paste-flow licence
//!     is NOT consulted as a fallback in this case, so an admin who
//!     deployed a corrupt file gets a clear "no Pro" signal rather than
//!     a quiet downgrade to whatever the user happened to have pasted.
//!   - Managed licence absent: orchestrator falls back to the existing
//!     keyring + cache flow exactly as before. No behaviour change for
//!     non-managed deployments.
//!
//! The on-disk file is the raw 96 bytes the verifier expects. No base64,
//! no JSON wrapper, no DPAPI. It is signed; the bytes are not secret in
//! the cryptographic sense and there is no value to encrypting them at
//! rest. (The cache file holds the same bytes plus an activation token,
//! which IS sensitive, hence the DPAPI wrap there.)

use std::path::PathBuf;

use crate::domain::licence::LicencePayload;
use crate::util::paths;
use crate::util::result::OrliqError;

use super::verify;

/// Filename the loader looks for inside the data dir. Matches the
/// website's `Content-Disposition: attachment; filename=...` slug only
/// in spirit: the website may serve `utterbar-licence-<id>.bin`, but
/// the IT admin renames it to this fixed name when dropping it into
/// the data dir. The doc page says so explicitly.
pub const MANAGED_LICENCE_FILENAME: &str = "licence.bin";

/// Path the loader inspects: `<data_dir>/licence.bin`. Surfaced as a
/// helper so the docs and the `where will I find it?` IT-admin
/// commands all read from the same source of truth.
pub fn managed_licence_path() -> Result<PathBuf, OrliqError> {
    let dir = paths::data_dir().map_err(|e| OrliqError::Licence {
        code: "internal".to_string(),
        message: format!("data dir resolve: {e}"),
    })?;
    Ok(dir.join(MANAGED_LICENCE_FILENAME))
}

/// Outcome of inspecting `<data_dir>/licence.bin` at startup.
#[derive(Debug)]
pub enum ManagedLicenceOutcome {
    /// No managed file present. The orchestrator falls back to the
    /// existing keyring + cache flow.
    Absent,
    /// File present and verified. Orchestrator should treat the machine
    /// as licensed under managed rules. The bytes are returned alongside
    /// the parsed payload so a future caller (audit log, support
    /// diagnostic dump, fingerprint pinning) can record the exact blob
    /// without re-reading the file.
    Loaded {
        payload: LicencePayload,
        /// Currently unread by the orchestrator (which only needs the
        /// parsed `payload` to synthesise `LicenceStatus`). Kept on
        /// the variant deliberately so future callers can include it
        /// in diagnostic dumps without re-reading the file. Marked
        /// `allow(dead_code)` to keep `-D warnings` clean.
        #[allow(dead_code)]
        signed_blob: Vec<u8>,
    },
    /// File present but failed verification (corrupt, wrong length,
    /// wrong signature, retired payload version). Orchestrator must
    /// NOT silently fall back to the cache: the explicit IT-admin
    /// intent of "this machine has a managed licence" is now
    /// contradicted by a bad file, so the safest read is "no Pro".
    /// The error is logged with enough detail for the admin to fix
    /// the file.
    Rejected {
        /// Carried for diagnostic / log-driven IT-admin troubleshooting.
        /// The orchestrator only inspects which variant was returned,
        /// not the inner error, so the field is currently unread by
        /// production code; the loader logs the same `OrliqError`
        /// before returning.
        #[allow(dead_code)]
        reason: OrliqError,
    },
}

/// Inspect the managed licence path. Pure function modulo a single
/// blocking file read; the orchestrator calls this synchronously at
/// `LicenceService::new()` (same place the cache is read).
pub fn try_load_managed() -> ManagedLicenceOutcome {
    let path = match managed_licence_path() {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!(error = ?e, "managed licence: data dir resolve failed");
            return ManagedLicenceOutcome::Absent;
        }
    };

    // Distinguish "file genuinely absent" from "file exists but
    // unreadable" so the absent branch produces a quiet info-level
    // log and the unreadable branch surfaces as a warning. Both
    // resolve to "no managed licence" in terms of state, but the
    // observability story differs.
    if !path.exists() {
        tracing::info!(path = %path.display(), "managed licence: file absent (using user/cache flow)");
        return ManagedLicenceOutcome::Absent;
    }

    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(e) => {
            tracing::warn!(
                path = %path.display(),
                error = ?e,
                "managed licence: file present but read failed"
            );
            return ManagedLicenceOutcome::Rejected {
                reason: OrliqError::Licence {
                    code: "managed_read_failed".to_string(),
                    message: format!("read {}: {e}", path.display()),
                },
            };
        }
    };

    match verify::verify_signed_blob(&bytes) {
        Ok(payload) => {
            tracing::info!(
                path = %path.display(),
                bytes = bytes.len(),
                version = payload.version,
                sku = payload.sku,
                key_id = payload.key_id,
                "managed licence: loaded and verified"
            );
            ManagedLicenceOutcome::Loaded {
                payload,
                signed_blob: bytes,
            }
        }
        Err(e) => {
            tracing::warn!(
                path = %path.display(),
                bytes = bytes.len(),
                error = ?e,
                "managed licence: file present but signature/payload rejected. Pro NOT granted."
            );
            ManagedLicenceOutcome::Rejected { reason: e }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `MANAGED_LICENCE_FILENAME` is referenced from the IT-deployment
    /// docs and from the website "how to install" copy. Anything that
    /// renames it must update both. This is a defensive constant pin.
    #[test]
    fn managed_licence_filename_is_stable() {
        assert_eq!(MANAGED_LICENCE_FILENAME, "licence.bin");
    }

    /// Path resolves under the project data dir. The data-dir helper
    /// returns `Err` only if `dirs::data_dir()` itself fails (almost
    /// never on Windows); we accept either branch so the test is
    /// robust on CI runners that don't have a normal data dir, but
    /// when it succeeds, the filename must be the constant.
    #[test]
    fn managed_licence_path_uses_constant_filename() {
        if let Ok(p) = managed_licence_path() {
            assert_eq!(
                p.file_name().and_then(|n| n.to_str()),
                Some(MANAGED_LICENCE_FILENAME),
                "managed licence path must end in {MANAGED_LICENCE_FILENAME}"
            );
        }
    }

    /// Absent file returns `Absent`. Achieved by pointing at a path
    /// that won't exist on a normal CI runner. We can't easily mock
    /// `paths::data_dir()` here without dependency injection, so this
    /// test exercises the public API and only asserts on its return
    /// shape rather than the internal log lines.
    #[test]
    fn try_load_managed_absent_when_no_data_dir_or_no_file() {
        // The function reads from `<data_dir>/licence.bin`. On a CI
        // runner with no Utterbar install present, the file should be
        // absent. We can't guarantee `data_dir()` succeeds on every
        // CI environment, but `Absent` is what both branches converge
        // on when there's nothing to load.
        match try_load_managed() {
            ManagedLicenceOutcome::Absent => {}
            ManagedLicenceOutcome::Loaded { .. } => {
                // A developer machine MIGHT have a real managed file
                // dropped at the data dir. That's a valid outcome too;
                // skip the assertion in that case rather than fail
                // someone's local cargo test run.
            }
            ManagedLicenceOutcome::Rejected { reason } => {
                panic!(
                    "unexpected Rejected outcome on a CI run with no managed file present: {reason:?}"
                );
            }
        }
    }
}
