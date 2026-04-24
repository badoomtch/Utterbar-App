//! Windows Credential Manager cache for the licence blob.
//!
//! Target name per docs/LICENCE_APP_HANDOFF §6: `Utterbar/Licence`. Value is a
//! JSON string; Windows encrypts it at rest via DPAPI. Using the `keyring`
//! crate v3 instead of calling `wincred` directly so a future port to another
//! platform just swaps out this module.
//!
//! We **never** store the fingerprint inside the signed blob itself — the blob
//! is server-signed and must round-trip untouched. The fingerprint is stashed
//! alongside so `/heartbeat` can re-send the exact bytes we activated with.
//!
//! ## v0.6.6: dual-cache (keyring + file) fallback
//!
//! Keyring v3 on Windows has been observed returning `Ok(())` from
//! `set_password` while the credential never actually appears in
//! Windows Credential Manager (`cmdkey /list`). Root cause unknown —
//! likely a Windows API quirk with forward-slashes in target names,
//! or an interaction with the process's integrity level under NSIS
//! installation. Until that's isolated, we write to BOTH the keyring
//! AND a plain JSON file at `%APPDATA%\Utterbar\licence.json`.
//!
//! Load order: keyring first (the canonical store), then file if
//! keyring returns `NoEntry`. The file is authoritative when keyring
//! is broken; keyring is authoritative when both are populated
//! (covers the "user deleted via cmdkey for testing" case).
//!
//! Security note: the licence blob is server-signed Ed25519 — tampering
//! with the file invalidates the signature, and `verify_signed_blob`
//! rejects it at load. There's nothing secret in there; the file is
//! stored in the user's roaming profile which is already per-user
//! access-controlled. Not encrypting at rest here is a deliberate
//! simplification; full DPAPI wrapping can be layered later if ever
//! needed (we have keyring for that already).

use anyhow::{Context, Result};
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use time::OffsetDateTime;

const SERVICE: &str = "Utterbar/Licence";
const ACCOUNT: &str = "default";
const FILE_NAME: &str = "licence.json";

/// Persistent record as stored in the OS keyring. Deliberately a flat JSON
/// object — the whole blob is small enough that we read/write it atomically on
/// every change, no partial updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedLicence {
    pub key_display: String,
    /// Base64 of the 96-byte signed blob.
    pub signed_blob_b64: String,
    pub activation_token: String,
    pub licence_id: u64,
    pub sku: String,
    pub max_devices: u16,
    pub fingerprint: String,
    /// RFC 3339 timestamp — next scheduled heartbeat. 30-day cadence per §4.4.
    pub next_heartbeat_at: String,
}

impl CachedLicence {
    /// Bump the heartbeat window 30 days from now. Called after a successful
    /// activate and after a 200 heartbeat response.
    pub fn bump_heartbeat(&mut self) {
        let next = OffsetDateTime::now_utc() + time::Duration::days(30);
        self.next_heartbeat_at = next
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default();
    }

    /// True when `now_utc() >= self.next_heartbeat_at`. Unparseable timestamps
    /// count as "due" — better to re-sync than silently skip.
    pub fn is_heartbeat_due(&self) -> bool {
        match OffsetDateTime::parse(
            &self.next_heartbeat_at,
            &time::format_description::well_known::Rfc3339,
        ) {
            Ok(next) => OffsetDateTime::now_utc() >= next,
            Err(_) => true,
        }
    }
}

pub struct LicenceCache;

impl LicenceCache {
    pub fn new() -> Self {
        Self
    }

    fn entry() -> Result<Entry> {
        Entry::new(SERVICE, ACCOUNT).context("create keyring entry")
    }

    /// Locate the fallback file path inside `%APPDATA%\Utterbar\`. Returns
    /// None on failure rather than propagating — the file path is advisory;
    /// the keyring is still canonical. Callers treat "no file path" the
    /// same as "file read failed".
    fn file_path() -> Option<PathBuf> {
        // `data_dir()` creates the directory if needed and returns the
        // `%APPDATA%\Utterbar\` canonical root.
        let dir = crate::util::paths::data_dir().ok()?;
        Some(dir.join(FILE_NAME))
    }

    fn load_from_file() -> Result<Option<CachedLicence>> {
        let Some(path) = Self::file_path() else {
            return Ok(None);
        };
        if !path.exists() {
            return Ok(None);
        }
        let json = fs::read_to_string(&path).context("read licence file")?;
        let rec: CachedLicence =
            serde_json::from_str(&json).context("decode cached licence file")?;
        Ok(Some(rec))
    }

    fn save_to_file(rec: &CachedLicence) -> Result<()> {
        let Some(path) = Self::file_path() else {
            anyhow::bail!("no data dir available for licence fallback file");
        };
        let json = serde_json::to_string(rec).context("encode cached licence for file")?;
        fs::write(&path, json).context("write licence file")?;
        Ok(())
    }

    fn clear_file() -> Result<()> {
        let Some(path) = Self::file_path() else {
            return Ok(());
        };
        if path.exists() {
            fs::remove_file(&path).context("remove licence file")?;
        }
        Ok(())
    }

    /// Returns `Ok(None)` when no record is stored. Any decode failure is
    /// surfaced as `Err` so the service can choose to clear the entry —
    /// tampering or corruption is treated the same as "no licence".
    ///
    /// Load order: keyring first, file fallback second. See module docs.
    pub fn load(&self) -> Result<Option<CachedLicence>> {
        // Keyring first.
        let entry = Self::entry()?;
        let from_keyring = match entry.get_password() {
            Ok(json) => {
                let rec: CachedLicence =
                    serde_json::from_str(&json).context("decode cached licence JSON")?;
                Some(rec)
            }
            Err(keyring::Error::NoEntry) => None,
            Err(e) => {
                // Don't fail the whole load on a keyring error — the file
                // fallback might still rescue us. Log noisily so the
                // keyring issue is visible.
                tracing::warn!(error = ?e, "keyring read failed; falling back to file cache");
                None
            }
        };
        if from_keyring.is_some() {
            return Ok(from_keyring);
        }

        // File fallback.
        match Self::load_from_file() {
            Ok(Some(rec)) => {
                tracing::info!(
                    "licence loaded from file fallback (keyring was empty or unavailable)"
                );
                Ok(Some(rec))
            }
            Ok(None) => Ok(None),
            Err(e) => {
                tracing::warn!(error = ?e, "licence file fallback read failed");
                Ok(None)
            }
        }
    }

    /// Writes to BOTH the keyring and the fallback file. Returns Ok even
    /// if the keyring write fails, as long as the file write succeeded —
    /// persistence is the guarantee, the mechanism is secondary. Both
    /// failing is a real error.
    pub fn save(&self, rec: &CachedLicence) -> Result<()> {
        let entry = Self::entry()?;
        let json = serde_json::to_string(rec).context("encode cached licence")?;
        let keyring_result = entry
            .set_password(&json)
            .context("write keyring entry");
        let file_result = Self::save_to_file(rec);

        match (keyring_result, file_result) {
            (Ok(()), Ok(())) => {
                tracing::info!("licence cached to keyring + file");
                Ok(())
            }
            (Ok(()), Err(fe)) => {
                // Keyring worked; file didn't. Tolerable — keyring is
                // canonical — but log it so we notice if keyring later
                // stops working too.
                tracing::warn!(error = ?fe, "licence saved to keyring but file write failed");
                Ok(())
            }
            (Err(ke), Ok(())) => {
                // Keyring failed silently (the v0.6.6 case) — the file
                // fallback saved us.
                tracing::warn!(
                    error = ?ke,
                    "keyring write failed; licence persisted via file fallback"
                );
                Ok(())
            }
            (Err(ke), Err(fe)) => {
                tracing::error!(keyring_err = ?ke, file_err = ?fe, "both licence stores failed");
                Err(ke)
            }
        }
    }

    pub fn clear(&self) -> Result<()> {
        let entry = Self::entry()?;
        let keyring_result = match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(anyhow::Error::new(e).context("delete keyring entry")),
        };
        let file_result = Self::clear_file();
        // Both should succeed; surface the keyring error first if it
        // failed, since that's the canonical store.
        keyring_result?;
        file_result?;
        Ok(())
    }
}

impl Default for LicenceCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heartbeat_due_when_timestamp_unparseable() {
        let rec = CachedLicence {
            key_display: "UB-0000-0000-0000-0000-0000".into(),
            signed_blob_b64: String::new(),
            activation_token: String::new(),
            licence_id: 1,
            sku: "utterbar-pro-lifetime".into(),
            max_devices: 3,
            fingerprint: "0".repeat(64),
            next_heartbeat_at: "not-a-timestamp".into(),
        };
        assert!(rec.is_heartbeat_due());
    }

    #[test]
    fn bump_heartbeat_writes_rfc3339() {
        let mut rec = CachedLicence {
            key_display: "UB-0000-0000-0000-0000-0000".into(),
            signed_blob_b64: String::new(),
            activation_token: String::new(),
            licence_id: 1,
            sku: "utterbar-pro-lifetime".into(),
            max_devices: 3,
            fingerprint: "0".repeat(64),
            next_heartbeat_at: String::new(),
        };
        rec.bump_heartbeat();
        // Parseable as RFC 3339 and at least ~29 days in the future — loose
        // bound avoids flakes from wall-clock drift during the test.
        let parsed = OffsetDateTime::parse(
            &rec.next_heartbeat_at,
            &time::format_description::well_known::Rfc3339,
        )
        .expect("RFC 3339");
        let delta = parsed - OffsetDateTime::now_utc();
        assert!(delta.whole_days() >= 29 && delta.whole_days() <= 31);
    }
}
