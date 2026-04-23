//! Licence system orchestrator.
//!
//! Wraps the four pure pieces (verify / keyfmt / api / cache) and the Windows
//! fingerprint into a single service the rest of the app talks to. Lives on
//! AppState, exposes three public verbs — `launch_check`, `activate`,
//! `deactivate` — plus a status snapshot the UI reads.
//!
//! Rules honoured (docs/LICENCE_APP_HANDOFF §11):
//! - Offline cache verification is the only path on the startup critical
//!   path. Network calls are either user-triggered (paste flow) or
//!   background-async (heartbeat).
//! - Signature failure == run as Free. No bypass.
//! - Absence of network is never treated as revocation. Only an explicit 410
//!   clears the cache.
//! - The raw signed blob and the fingerprint are never exposed in
//!   `LicenceStatus`. Only the device list (which includes fingerprints per
//!   the server contract, for UI "this device" highlighting) carries hex
//!   fingerprints, which is fine — the server already sends them down.

pub mod api;
pub mod cache;
pub mod keyfmt;
pub mod verify;

use std::sync::Arc;

use base64::{engine::general_purpose::STANDARD, Engine as _};
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;

use crate::domain::licence::{LicenceDevice, LicenceStatus};
use crate::util::result::OrliqError;

pub use api::{ApiDevice, LicenceApi};
pub use cache::{CachedLicence, LicenceCache};

/// Frontend event slug — payload is `LicenceStatus` JSON. Emitted on every
/// state change (activate, deactivate, revoke, launch-check result).
pub const EVT_LICENCE_CHANGED: &str = "licence_changed";

pub struct LicenceService {
    api: Arc<LicenceApi>,
    cache: LicenceCache,
    /// Computed once at service construction. Failing to compute a
    /// fingerprint leaves this `None`; `activate` then returns an `internal`
    /// error rather than silently fingerprinting as zeros.
    fingerprint: Option<String>,
    device_name: String,
    /// Live status. Updated from cache on launch, then from paste /
    /// heartbeat results.
    status: RwLock<LicenceStatus>,
}

impl LicenceService {
    pub fn new() -> Self {
        // Compute the fingerprint synchronously at startup. It's cheap (two
        // registry reads + one adapter enumeration) and the value feeds every
        // server call — spreading it across an `OnceCell` would only add
        // failure modes.
        let (fingerprint, device_name) = {
            #[cfg(windows)]
            {
                let fp = crate::platform::windows::fingerprint::compute_fingerprint()
                    .inspect_err(|e| tracing::error!(error = ?e, "fingerprint failed"))
                    .ok();
                let name = crate::platform::windows::fingerprint::device_name();
                (fp, name)
            }
            #[cfg(not(windows))]
            {
                // Utterbar is Windows-first; we still compile on macOS/Linux so
                // editors + CI can run clippy/tests. The licence system is a
                // no-op on non-Windows — activate() returns an internal error
                // explaining the platform is unsupported.
                (None, "Unknown Device".to_string())
            }
        };

        Self {
            api: LicenceApi::shared(),
            cache: LicenceCache::new(),
            fingerprint,
            device_name,
            status: RwLock::new(LicenceStatus::Free),
        }
    }

    pub fn shared() -> Arc<Self> {
        Arc::new(Self::new())
    }

    /// Snapshot of current state. Cheap — cloned `LicenceStatus` enum.
    pub async fn status(&self) -> LicenceStatus {
        self.status.read().await.clone()
    }

    /// Launch-time entry point. Reads the cache, verifies the blob offline,
    /// emits the status event, and — if the heartbeat window has elapsed —
    /// kicks a background heartbeat. Never blocks on the network.
    pub async fn launch_check(&self, handle: AppHandle) {
        let Ok(maybe) = self.cache.load() else {
            tracing::warn!("licence cache read failed; running as Free");
            let _ = self.cache.clear();
            self.set_and_emit(&handle, LicenceStatus::Free).await;
            return;
        };
        let Some(cached) = maybe else {
            self.set_and_emit(&handle, LicenceStatus::Free).await;
            return;
        };

        // Verify the blob offline. Any failure == clear cache, Free.
        let blob = match STANDARD.decode(&cached.signed_blob_b64) {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!(error = ?e, "cached blob not base64; clearing");
                let _ = self.cache.clear();
                self.set_and_emit(&handle, LicenceStatus::Free).await;
                return;
            }
        };
        if let Err(e) = verify::verify_signed_blob(&blob) {
            tracing::warn!(error = ?e, "cached licence signature invalid; clearing");
            let _ = self.cache.clear();
            self.set_and_emit(&handle, LicenceStatus::Free).await;
            return;
        }

        // Valid. Surface Pro state, even if the heartbeat is due.
        let status = self.status_from_cache(&cached, Vec::new());
        self.set_and_emit(&handle, status).await;

        if cached.is_heartbeat_due() {
            // Background heartbeat — copy out what the spawn needs so we don't
            // borrow `self` across the await. On revoke we clear via a fresh
            // LicenceCache handle and re-emit `Free` through the AppHandle; we
            // don't need to mutate the service's live `status` because
            // launch_check already emitted the `Pro` state and the UI is
            // subscribed to `licence_changed`.
            let svc_api = self.api.clone();
            let fingerprint = self.fingerprint.clone();
            let cache = LicenceCache::new();
            let cached_c = cached.clone();
            let handle_c = handle.clone();
            tauri::async_runtime::spawn(async move {
                let Some(fp) = fingerprint else {
                    return;
                };
                match svc_api.heartbeat(&cached_c.key_display, &fp).await {
                    Ok(_) => {
                        let mut updated = cached_c.clone();
                        updated.bump_heartbeat();
                        if let Err(e) = cache.save(&updated) {
                            tracing::warn!(error = ?e, "failed to persist heartbeat bump");
                        }
                    }
                    Err(OrliqError::Licence { code, message }) if code == "revoked" => {
                        tracing::warn!(message, "licence revoked by server; clearing");
                        let _ = cache.clear();
                        // Let the UI hear about it.
                        let _ = handle_c.emit(EVT_LICENCE_CHANGED, LicenceStatus::Free);
                    }
                    Err(e) => {
                        // Network / transient — per §11, never treat as revocation.
                        tracing::info!(error = ?e, "heartbeat deferred to next launch");
                    }
                }
            });
        }
    }

    /// Paste flow. `display_key` may be any of the accepted input forms
    /// (display, canonical, with or without UB prefix). Performs: normalise →
    /// fetch → verify → activate → cache → set status.
    pub async fn activate(
        &self,
        handle: AppHandle,
        display_key: &str,
    ) -> Result<LicenceStatus, OrliqError> {
        let canonical = keyfmt::normalise(display_key)?;
        let display = keyfmt::format_display(&canonical);

        let fingerprint = self
            .fingerprint
            .clone()
            .ok_or_else(|| OrliqError::Licence {
                code: "internal".to_string(),
                message: "device fingerprint unavailable — licence activation requires Windows"
                    .to_string(),
            })?;

        // 1. /fetch — returns the signed blob.
        let fetched = self.api.fetch(&display).await?;

        // 2. Verify signature offline before we even think about caching.
        let blob = STANDARD
            .decode(&fetched.signed_blob)
            .map_err(|e| OrliqError::Licence {
                code: "malformed".to_string(),
                message: format!("server returned non-base64 blob: {e}"),
            })?;
        verify::verify_signed_blob(&blob)?;

        // 3. /activate — registers this device.
        let activated = self
            .api
            .activate(&display, &fingerprint, &self.device_name)
            .await?;

        // 4. Persist. This is the point-of-no-return for this key; after a
        //    successful activate the server considers the device registered
        //    even if we crash before writing the cache, so we write first and
        //    let the user retry if the keyring itself misbehaves.
        let mut cached = CachedLicence {
            key_display: display.clone(),
            signed_blob_b64: fetched.signed_blob.clone(),
            activation_token: activated.activation_token.clone(),
            licence_id: fetched.licence_id,
            sku: fetched.sku.clone(),
            max_devices: fetched.max_devices,
            fingerprint: fingerprint.clone(),
            next_heartbeat_at: String::new(),
        };
        cached.bump_heartbeat();
        self.cache.save(&cached).map_err(|e| OrliqError::Licence {
            code: "internal".to_string(),
            message: format!("failed to persist licence: {e}"),
        })?;

        // 5. Surface status.
        let devices = activated.devices.into_iter().map(device_from_api).collect();
        let status = self.status_from_cache(&cached, devices);
        self.set_and_emit(&handle, status.clone()).await;
        Ok(status)
    }

    /// Remove this device from the licence and clear the local cache.
    /// Failures on the network path are non-fatal for the local state — the
    /// user clicked "remove this device", they get a Free state regardless,
    /// and they can reclaim the slot via utterbar.com/account if the server
    /// call didn't complete.
    pub async fn deactivate(&self, handle: AppHandle) -> Result<LicenceStatus, OrliqError> {
        let current = self.cache.load().map_err(|e| OrliqError::Licence {
            code: "internal".to_string(),
            message: format!("read cache: {e}"),
        })?;

        if let Some(rec) = current {
            if let Some(fp) = self.fingerprint.clone() {
                if let Err(e) = self.api.deactivate(&rec.key_display, &fp).await {
                    tracing::warn!(error = ?e, "server deactivate failed; clearing local anyway");
                }
            }
        }

        let _ = self.cache.clear();
        self.set_and_emit(&handle, LicenceStatus::Free).await;
        Ok(LicenceStatus::Free)
    }

    /// Build a `LicenceStatus::Pro` from a cached record. Devices list is
    /// passed in because it's only available fresh from `/activate`;
    /// launch-path reconstruction starts with an empty list and the UI can
    /// re-fetch via a dedicated command later if needed.
    fn status_from_cache(
        &self,
        cached: &CachedLicence,
        devices: Vec<LicenceDevice>,
    ) -> LicenceStatus {
        LicenceStatus::Pro {
            display_key: cached.key_display.clone(),
            licence_id: cached.licence_id.to_string(),
            sku: cached.sku.clone(),
            issued_at_ms: String::new(), // not round-tripped; UI shows "active since" from server on next /fetch
            max_devices: cached.max_devices,
            devices,
        }
    }

    async fn set_and_emit(&self, handle: &AppHandle, new_status: LicenceStatus) {
        {
            let mut w = self.status.write().await;
            *w = new_status.clone();
        }
        if let Err(e) = handle.emit(EVT_LICENCE_CHANGED, new_status) {
            tracing::warn!(error = ?e, "failed to emit licence_changed");
        }
    }
}

impl Default for LicenceService {
    fn default() -> Self {
        Self::new()
    }
}

fn device_from_api(d: ApiDevice) -> LicenceDevice {
    LicenceDevice {
        fingerprint: d.fingerprint,
        device_name: d.device_name,
        first_seen: d.first_seen,
        this_device: d.this_device,
    }
}
