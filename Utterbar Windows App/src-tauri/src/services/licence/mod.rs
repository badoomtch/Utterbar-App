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
pub mod managed;
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

/// Outcome of inspecting `<data_dir>/licence.bin` at bootstrap, frozen
/// for the lifetime of `LicenceService`. The file is consulted exactly
/// once at startup; changes require an app restart (same contract as
/// the rest of the policy stack).
///
/// Three arms instead of a `bool` so the rejected case can block paste
/// activation independently of whether managed Pro is active. Without
/// this distinction a corrupt/unreadable managed file would fail the
/// bootstrap closed (good — no Pro), but a determined user could then
/// paste their own licence and grant themselves Pro, which is exactly
/// the override the IT admin's deployment was supposed to prevent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagedLicenceState {
    /// No `licence.bin` on disk. The cache / paste flow is the
    /// authoritative source — byte-for-byte unchanged from the
    /// pre-B2B-sprint behaviour.
    Absent,
    /// File present and verified. Synthesises a Pro `LicenceStatus`
    /// at bootstrap and blocks paste/deactivate so the IT-admin
    /// intent isn't accidentally overwritten.
    Loaded,
    /// File present but failed verification (corrupt, wrong length,
    /// wrong signature, unreadable, retired payload version). Status
    /// is Free — we deliberately do NOT fall back to the cache —
    /// AND paste/deactivate are blocked. The explicit "this machine
    /// is managed" intent is contradicted by a bad file; the safe
    /// read is "no Pro on this machine until the admin fixes the
    /// file", not "let the user paste whatever they have".
    Rejected,
}

impl ManagedLicenceState {
    /// True when the bootstrap loaded a verified managed licence and
    /// the machine is running as managed Pro. Backs the
    /// `licence_is_managed` IPC so the Settings → Licence panel
    /// renders the "Managed by your administrator" notice. Returns
    /// false for `Rejected`: the machine is NOT licensed under
    /// managed rules in that state, so the managed UI shouldn't
    /// imply otherwise.
    pub fn is_loaded(self) -> bool {
        matches!(self, ManagedLicenceState::Loaded)
    }

    /// True when a managed licence file is present on disk in any
    /// state — verified or rejected. Used by `activate` and
    /// `deactivate` to refuse user-driven licence changes. The
    /// rationale is that an IT admin who deployed *anything* to
    /// `<data_dir>/licence.bin` has signalled "this machine is
    /// managed"; the right response to a corrupt file is to fix the
    /// file, not to let the user paste a personal key as an
    /// override.
    pub fn activation_blocked(self) -> bool {
        matches!(
            self,
            ManagedLicenceState::Loaded | ManagedLicenceState::Rejected
        )
    }
}

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
    /// Outcome of the managed-licence file probe at bootstrap. Frozen
    /// at construction; see `ManagedLicenceState` for the three-arm
    /// semantics. Mostly read via `is_managed()` and
    /// `activation_blocked_by_managed_file()` rather than this raw
    /// field.
    managed_state: ManagedLicenceState,
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

        // Bootstrap precedence (B2B sprint, fleet licence delivery):
        //   1. Managed `<data_dir>/licence.bin`. If present and verifies,
        //      it wins over any cached / paste licence. If present but
        //      invalid, fail closed: the explicit IT-admin "this machine
        //      is managed" intent is contradicted by a bad file, so we
        //      do NOT fall back to the cache. Status stays Free, the
        //      frontend shows the licence page in its non-Pro state, and
        //      the IT admin has a clear log line saying why.
        //   2. Keyring cache (existing behaviour). Same synchronous
        //      verify-or-clear flow as before, unchanged.
        //
        // Doing the initial verify synchronously at bootstrap is ~0ms
        // for a no-licence user (one path-stat miss) and ~1ms for a
        // managed or cached user (one read + one ed25519-dalek verify).
        // No network calls here — `launch_check` from the background
        // task still handles the 30-day heartbeat for cache-flow users.
        let cache = LicenceCache::new();
        let (initial_status, managed_state) = match managed::try_load_managed() {
            managed::ManagedLicenceOutcome::Loaded { payload, .. } => {
                // Synthesise a Pro status from the verified payload.
                // Managed deployments don't have an activation token,
                // a server-issued device list, or a max_devices count
                // (those come from the activate flow which managed
                // licences bypass entirely). The frontend renders this
                // state as "Managed by your administrator" and hides
                // the device-list / activation form.
                let status = LicenceStatus::Pro {
                    display_key: "Managed".to_string(),
                    licence_id: payload.key_id.to_string(),
                    sku: "utterbar-managed".to_string(),
                    issued_at_ms: payload.issued_at_ms.to_string(),
                    max_devices: 0,
                    devices: Vec::new(),
                };
                (status, ManagedLicenceState::Loaded)
            }
            managed::ManagedLicenceOutcome::Rejected { .. } => {
                // Logged with detail by `try_load_managed`. We do NOT
                // consult the cache here: an admin who deployed a
                // bad managed file gets a clear "no Pro" signal
                // rather than a quiet downgrade to whatever the user
                // happened to paste. Paste activation is also
                // blocked downstream (see `activate`) so the user
                // can't override the bad-file state from the UI.
                (LicenceStatus::Free, ManagedLicenceState::Rejected)
            }
            managed::ManagedLicenceOutcome::Absent => {
                // No managed file present: fall back to the existing
                // keyring + cache flow, byte-for-byte unchanged from
                // pre-B2B-sprint behaviour.
                let status = match cache.load() {
                    Ok(Some(cached)) => match STANDARD.decode(&cached.signed_blob_b64) {
                        Ok(blob) => match verify::verify_signed_blob(&blob) {
                            Ok(_) => {
                                let s = LicenceStatus::Pro {
                                    display_key: cached.key_display.clone(),
                                    licence_id: cached.licence_id.to_string(),
                                    sku: cached.sku.clone(),
                                    issued_at_ms: String::new(),
                                    max_devices: cached.max_devices,
                                    devices: Vec::new(),
                                };
                                tracing::info!("licence restored from keyring cache");
                                s
                            }
                            Err(e) => {
                                tracing::warn!(error = ?e, "cached licence signature invalid at bootstrap; clearing");
                                let _ = cache.clear();
                                LicenceStatus::Free
                            }
                        },
                        Err(e) => {
                            tracing::warn!(error = ?e, "cached blob not base64 at bootstrap; clearing");
                            let _ = cache.clear();
                            LicenceStatus::Free
                        }
                    },
                    Ok(None) => LicenceStatus::Free,
                    Err(e) => {
                        tracing::warn!(error = ?e, "licence cache read failed at bootstrap; running as Free");
                        LicenceStatus::Free
                    }
                };
                (status, ManagedLicenceState::Absent)
            }
        };

        Self {
            api: LicenceApi::shared(),
            cache,
            fingerprint,
            device_name,
            status: RwLock::new(initial_status),
            managed_state,
        }
    }

    /// True when the active Pro status came from a verified managed
    /// (fleet) licence file. Returns false in the rejected case
    /// (file present but bad signature / unreadable / wrong length)
    /// because the machine is NOT licensed under managed rules in
    /// that state — it's running as Free with paste blocked. The
    /// frontend reads this via the `licence_is_managed` Tauri
    /// command to swap the activation form for a "managed by your
    /// administrator" notice.
    pub fn is_managed(&self) -> bool {
        self.managed_state.is_loaded()
    }

    /// True if a managed licence file exists on disk in any state —
    /// verified or rejected. Used by `activate` and `deactivate` to
    /// refuse user-driven licence changes; the rejected branch is
    /// what stops a user pasting a personal key as an override when
    /// the IT-admin-deployed file is corrupt.
    pub fn activation_blocked_by_managed_file(&self) -> bool {
        self.managed_state.activation_blocked()
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
        // Managed (fleet) licence path. `Loaded` already populated
        // the Pro status at bootstrap; re-emit and stop. `Rejected`
        // already populated Free at bootstrap; re-emit and stop —
        // we explicitly do NOT consult the cache here, because the
        // cache fallback would defeat the IT-admin "this machine
        // is managed" intent. Only `Absent` runs the cache flow.
        match self.managed_state {
            ManagedLicenceState::Loaded => {
                let s = self.status.read().await.clone();
                tracing::info!(
                    "licence: managed mode active at launch; skipping cache + heartbeat"
                );
                let _ = handle.emit(EVT_LICENCE_CHANGED, s);
                return;
            }
            ManagedLicenceState::Rejected => {
                tracing::warn!(
                    "licence: managed file present but invalid; running as Free with paste blocked"
                );
                let _ = handle.emit(EVT_LICENCE_CHANGED, LicenceStatus::Free);
                return;
            }
            ManagedLicenceState::Absent => {}
        }

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
        // Managed (fleet) licence wins regardless of whether the
        // file verified. Three-arm semantics:
        //   * Absent  — proceed with the paste flow (fall through).
        //   * Loaded  — managed Pro is already active; user has
        //               nothing to activate.
        //   * Rejected — file is present but bad. We MUST also block
        //                here, otherwise a user could paste their
        //                own licence and grant themselves Pro on a
        //                machine the IT admin tried to manage. The
        //                fix is "delete the file or push a valid
        //                one", not "let the user override".
        if self.activation_blocked_by_managed_file() {
            let message = match self.managed_state {
                ManagedLicenceState::Loaded => {
                    "This machine is managed by your administrator. \
                     Activation is not available; the licence is delivered \
                     as a file."
                }
                ManagedLicenceState::Rejected => {
                    "A managed licence file is present but invalid. \
                     Contact your IT administrator."
                }
                ManagedLicenceState::Absent => {
                    unreachable!("activation_blocked_by_managed_file() returned true for Absent")
                }
            };
            return Err(OrliqError::Licence {
                code: "managed_licence_active".to_string(),
                message: message.to_string(),
            });
        }

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
        // Managed (fleet) licence: deactivation is not user-controlled.
        // The IT admin removes `licence.bin` from disk; on the next
        // launch the bootstrap finds it absent and falls back to the
        // cache flow. Same three-arm semantics as `activate`: we
        // block on both `Loaded` (managed Pro active) and `Rejected`
        // (bad file present) so a user can't clear a stale cache to
        // sidestep the managed posture.
        if self.activation_blocked_by_managed_file() {
            let message = match self.managed_state {
                ManagedLicenceState::Loaded => {
                    "This machine is managed by your administrator. \
                     Removal is handled by deleting the licence file."
                }
                ManagedLicenceState::Rejected => {
                    "A managed licence file is present but invalid. \
                     Contact your IT administrator."
                }
                ManagedLicenceState::Absent => {
                    unreachable!("activation_blocked_by_managed_file() returned true for Absent")
                }
            };
            return Err(OrliqError::Licence {
                code: "managed_licence_active".to_string(),
                message: message.to_string(),
            });
        }

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

#[cfg(test)]
mod managed_state_tests {
    //! Pure semantics tests for `ManagedLicenceState` and the
    //! `activation_blocked` / `is_loaded` helpers. Constructing a
    //! full `LicenceService` here would pull in the keyring, the
    //! fingerprint compute, and the registry probe — none of which
    //! the enum logic needs. Pinning the helpers at this layer
    //! catches any future refactor that confuses the two predicates
    //! (e.g. accidentally letting `Rejected` through as "not
    //! managed, paste away") without paying for a heavy fixture.
    use super::ManagedLicenceState;

    /// `Absent` must NOT block activation. This is the fall-through
    /// case where a normal user without a managed deployment can
    /// paste their licence as before.
    #[test]
    fn absent_does_not_block_activation() {
        assert!(!ManagedLicenceState::Absent.activation_blocked());
        assert!(!ManagedLicenceState::Absent.is_loaded());
    }

    /// `Loaded` must block activation: managed Pro is already
    /// active, the user has nothing to activate.
    #[test]
    fn loaded_blocks_activation() {
        assert!(ManagedLicenceState::Loaded.activation_blocked());
        assert!(ManagedLicenceState::Loaded.is_loaded());
    }

    /// `Rejected` must block activation. This is the load-bearing
    /// fix for the review finding: a corrupt managed file used to
    /// leave the bool `is_managed = false`, which let a user paste
    /// their own licence and grant themselves Pro on a machine the
    /// IT admin had tried to manage. The new helper closes that
    /// gap by returning true for both `Loaded` and `Rejected`.
    #[test]
    fn rejected_blocks_activation() {
        assert!(ManagedLicenceState::Rejected.activation_blocked());
    }

    /// `Rejected` is NOT `is_loaded`. The Settings → Licence panel
    /// reads `licence_is_managed` (which delegates to `is_loaded`)
    /// to decide whether to render the "Managed by your
    /// administrator" notice. Showing that notice for a rejected
    /// file would be misleading: the machine is running as Free,
    /// not as managed Pro. The user-visible state for `Rejected`
    /// is the standard Free panel; the activation form on it is
    /// what `activation_blocked` disables.
    #[test]
    fn rejected_is_not_loaded() {
        assert!(!ManagedLicenceState::Rejected.is_loaded());
    }

    /// The two predicates have the documented relationship:
    /// `is_loaded` ⊂ `activation_blocked`. Anything `is_loaded`
    /// MUST be `activation_blocked`; the converse is not required.
    #[test]
    fn is_loaded_implies_activation_blocked() {
        for state in [
            ManagedLicenceState::Absent,
            ManagedLicenceState::Loaded,
            ManagedLicenceState::Rejected,
        ] {
            if state.is_loaded() {
                assert!(
                    state.activation_blocked(),
                    "is_loaded must imply activation_blocked: {state:?}"
                );
            }
        }
    }
}
