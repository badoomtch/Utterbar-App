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

use anyhow::{Context, Result};
use keyring::Entry;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

const SERVICE: &str = "Utterbar/Licence";
const ACCOUNT: &str = "default";

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

    /// Returns `Ok(None)` when no record is stored. Any decode failure is
    /// surfaced as `Err` so the service can choose to clear the entry —
    /// tampering or corruption is treated the same as "no licence".
    pub fn load(&self) -> Result<Option<CachedLicence>> {
        let entry = Self::entry()?;
        match entry.get_password() {
            Ok(json) => {
                let rec: CachedLicence =
                    serde_json::from_str(&json).context("decode cached licence JSON")?;
                Ok(Some(rec))
            }
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(anyhow::Error::new(e).context("read keyring entry")),
        }
    }

    pub fn save(&self, rec: &CachedLicence) -> Result<()> {
        let entry = Self::entry()?;
        let json = serde_json::to_string(rec).context("encode cached licence")?;
        entry.set_password(&json).context("write keyring entry")?;
        Ok(())
    }

    pub fn clear(&self) -> Result<()> {
        let entry = Self::entry()?;
        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(anyhow::Error::new(e).context("delete keyring entry")),
        }
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
