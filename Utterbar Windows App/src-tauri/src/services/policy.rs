//! Group-policy / managed-deployment overrides.
//!
//! For Business and Enterprise deployments, an IT admin needs to be able to
//! pin a small set of behaviours regardless of what the per-user
//! `settings.json` says. The classical Windows shape for this is a registry
//! key under `HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Utterbar` (machine-wide,
//! requires admin to set) with `HKEY_CURRENT_USER\SOFTWARE\Policies\Utterbar`
//! as a per-user fallback. HKLM wins on conflict; if a value is absent from
//! both, the per-user setting from `settings.json` is used unchanged.
//!
//! This module is a tiny, focused surface. It reads the registry once at
//! boot and exposes a `Policy` value that the settings layer overlays on
//! every `snapshot()`. Read-only, no caching beyond the value already
//! captured at boot. To pick up a newly-pinned policy the user must
//! restart the app — same as Windows itself for most policy changes,
//! and within the expectation of any IT admin running an update.
//!
//! ## Currently pinnable values
//!
//! - `UpdateChannel` (REG_SZ): `stable` or `beta`. Pins
//!   `Settings.advanced.update_channel`.
//! - `CheckUpdatesAutomatically` (REG_DWORD): 0 or 1. Pins
//!   `Settings.general.check_updates_automatically`.
//! - `BlockModelDownloads` (REG_DWORD): 0 or 1. Surfaces a hard refusal
//!   in the model installer when 1; the catalogue UI shows the override
//!   so the user understands why a download won't happen.
//! - `BlockSystemAudio` (REG_DWORD): 0 or 1. Forces
//!   `Settings.audio.capture_source` back to `Microphone` regardless of
//!   what the user picks. Compliance ask for environments where call /
//!   meeting audio must not be captured (e.g. healthcare, finance).
//! - `BlockHistory` (REG_DWORD): 0 or 1. When 1, no new history rows
//!   are persisted. Existing rows are NOT deleted — the IT admin's
//!   intent is "stop collecting more", not "destroy the past". Set
//!   together with `BlockHistoryExport` if the goal is total lock-down.
//! - `BlockHistoryExport` (REG_DWORD): 0 or 1. When 1, the export-as-
//!   JSON command refuses. History remains visible in the UI; this is
//!   the narrower control for environments where the user is allowed
//!   to consult their own transcripts but not move them off-device.
//! - `BlockTelemetry` (REG_DWORD): 0 or 1. When 1, the local analytics
//!   rollup (words / dictations / saved-seconds in
//!   `analytics_daily`) does not bump on a successful dictation. There
//!   is no remote telemetry endpoint — this gates the on-device SQLite
//!   rollup that powers the Dashboard, for environments where even
//!   local usage counters are out of scope.
//!
//! ## Adding a new pinnable value
//!
//! Three steps:
//!   1. Add a field to `Policy`.
//!   2. Read it in `read_from_registry()`.
//!   3. Apply it in `Policy::apply_to(...)` against the `Settings`
//!      struct.
//!
//! Keep this list small. Every pinnable value is a maintenance burden
//! and a UI explanation; the only ones worth adding are values an IT
//! admin would credibly want to control across a fleet.

use crate::domain::{settings::CaptureSource, Settings};

#[cfg(windows)]
const POLICY_SUBKEY: &str = r"SOFTWARE\Policies\Utterbar";

/// The set of policy-pinned values resolved at app boot. `None` on a
/// field means "no policy override; keep whatever's in settings.json".
#[derive(Debug, Clone, Default)]
pub struct Policy {
    /// `stable` or `beta`.
    pub update_channel: Option<String>,
    /// Pins the auto-update-on-launch toggle.
    pub check_updates_automatically: Option<bool>,
    /// When true, the model catalogue refuses any new download. Used
    /// for environments where models must come from an internal
    /// mirror, not the public release URLs.
    pub block_model_downloads: Option<bool>,
    /// When true, forces `Settings.audio.capture_source` to
    /// `Microphone` regardless of the user's selection. Compliance
    /// ask for environments that must not capture meeting / call
    /// audio (healthcare, finance, regulated call centres).
    pub block_system_audio: Option<bool>,
    /// When true, new history rows are not persisted. Existing
    /// rows are left untouched — the policy is forward-looking
    /// ("stop collecting more"), not retrospective. The Settings
    /// → Privacy panel shows a "Disabled by your administrator"
    /// hint next to the History controls so the user knows why
    /// nothing new is appearing.
    pub block_history: Option<bool>,
    /// When true, the export-as-JSON command refuses. History
    /// stays visible in the UI; only the "move it off-device"
    /// surface is disabled. If `block_history` is also set the
    /// export refuses too (no rows worth exporting), but
    /// `block_history_export` alone is the right pick when the
    /// IT admin wants the user to be able to read their own
    /// transcripts but not redistribute them.
    pub block_history_export: Option<bool>,
    /// When true, the local analytics rollup
    /// (`AnalyticsService::record_dictation`) is skipped on a
    /// successful dictation. The Dashboard surfaces stop
    /// accumulating new data; existing rollup rows are not
    /// deleted. There is no remote telemetry — this gates the
    /// on-device SQLite rollup only.
    pub block_telemetry: Option<bool>,
}

impl Policy {
    /// Resolve from registry on Windows. No-op (returns Default) on
    /// other platforms; we only build for Windows in production but
    /// the cross-compilation must succeed.
    pub fn load() -> Self {
        #[cfg(windows)]
        {
            read_from_registry()
        }
        #[cfg(not(windows))]
        {
            Self::default()
        }
    }

    /// Convenience for the model service: did the IT admin disable
    /// model downloads on this machine?
    pub fn model_downloads_blocked(&self) -> bool {
        self.block_model_downloads.unwrap_or(false)
    }

    /// True if this machine should not persist new history rows.
    pub fn history_blocked(&self) -> bool {
        self.block_history.unwrap_or(false)
    }

    /// True if the export surface is disabled. Returns true when
    /// either `BlockHistoryExport` is explicitly set OR
    /// `BlockHistory` is set (no point exporting a frozen set, and
    /// "block history" is the strict superset of "block export").
    pub fn history_export_blocked(&self) -> bool {
        self.block_history_export.unwrap_or(false) || self.history_blocked()
    }

    /// True if the local analytics rollup is disabled. Used by the
    /// dictation success path to skip the `record_dictation` call.
    pub fn telemetry_blocked(&self) -> bool {
        self.block_telemetry.unwrap_or(false)
    }

    /// Apply the policy to a freshly-loaded Settings. Mutates only the
    /// fields the policy pins; everything else is left as the user
    /// saved it. Called from `SettingsStore::snapshot()` on every read,
    /// not just at boot, so a user clicking "Stable" in the UI when
    /// the policy pins them to "Beta" gets their click ignored on the
    /// very next read.
    pub fn apply_to(&self, s: &mut Settings) {
        if let Some(channel) = &self.update_channel {
            // Validate against the known set. An IT admin who typoed
            // `stabel` shouldn't break the app; we just ignore the bad
            // value and the user-side default takes over.
            if matches!(channel.as_str(), "stable" | "beta") {
                s.advanced.update_channel = channel.clone();
            }
        }
        if let Some(check) = self.check_updates_automatically {
            s.general.check_updates_automatically = check;
        }
        if let Some(true) = self.block_system_audio {
            // Force back to mic-only on every read. If the user has
            // already picked System or Mixed in the UI we silently
            // ignore that choice — the click on the next read produces
            // the policy-pinned value, same shape as update_channel.
            s.audio.capture_source = CaptureSource::Microphone;
        }
        // `block_model_downloads` is consumed directly by the model
        // service via `Policy::model_downloads_blocked()` — it gates
        // runtime behaviour, not a user preference, so it has no
        // Settings field to mirror into.
    }

    /// True if any policy is pinned. Used by the UI to show a small
    /// "managed by your IT admin" hint next to affected controls
    /// rather than letting the user click and have the change silently
    /// ignored.
    pub fn is_active(&self) -> bool {
        self.update_channel.is_some()
            || self.check_updates_automatically.is_some()
            || self.block_model_downloads.is_some()
            || self.block_system_audio.is_some()
            || self.block_history.is_some()
            || self.block_history_export.is_some()
            || self.block_telemetry.is_some()
    }
}

#[cfg(windows)]
fn read_from_registry() -> Policy {
    use winreg::enums::*;
    use winreg::RegKey;

    let mut policy = Policy::default();

    // HKEY_LOCAL_MACHINE first (machine-wide, needs admin to set).
    // Anything found here wins over per-user.
    if let Ok(key) = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(POLICY_SUBKEY) {
        merge_keys(&key, &mut policy);
    }

    // HKEY_CURRENT_USER as a fallback (per-user, doesn't need admin).
    // Useful for testing on a dev machine without an elevated shell.
    // HKLM wins on conflict because we already merged it first; only
    // values still `None` after the HKLM pass get filled here.
    if let Ok(key) = RegKey::predef(HKEY_CURRENT_USER).open_subkey(POLICY_SUBKEY) {
        merge_keys(&key, &mut policy);
    }

    if policy.is_active() {
        tracing::info!(
            update_channel = ?policy.update_channel,
            check_updates_auto = ?policy.check_updates_automatically,
            block_model_downloads = ?policy.block_model_downloads,
            block_system_audio = ?policy.block_system_audio,
            block_history = ?policy.block_history,
            block_history_export = ?policy.block_history_export,
            block_telemetry = ?policy.block_telemetry,
            "policy: managed values active"
        );
    }

    policy
}

#[cfg(windows)]
fn merge_keys(key: &winreg::RegKey, policy: &mut Policy) {
    // String value. Only set if not already set (HKLM-first wins).
    if policy.update_channel.is_none() {
        if let Ok(v) = key.get_value::<String, _>("UpdateChannel") {
            policy.update_channel = Some(v.trim().to_lowercase());
        }
    }
    // DWORD values. winreg returns u32; we treat 0 as false and any
    // non-zero as true.
    if policy.check_updates_automatically.is_none() {
        if let Ok(v) = key.get_value::<u32, _>("CheckUpdatesAutomatically") {
            policy.check_updates_automatically = Some(v != 0);
        }
    }
    if policy.block_model_downloads.is_none() {
        if let Ok(v) = key.get_value::<u32, _>("BlockModelDownloads") {
            policy.block_model_downloads = Some(v != 0);
        }
    }
    if policy.block_system_audio.is_none() {
        if let Ok(v) = key.get_value::<u32, _>("BlockSystemAudio") {
            policy.block_system_audio = Some(v != 0);
        }
    }
    if policy.block_history.is_none() {
        if let Ok(v) = key.get_value::<u32, _>("BlockHistory") {
            policy.block_history = Some(v != 0);
        }
    }
    if policy.block_history_export.is_none() {
        if let Ok(v) = key.get_value::<u32, _>("BlockHistoryExport") {
            policy.block_history_export = Some(v != 0);
        }
    }
    if policy.block_telemetry.is_none() {
        if let Ok(v) = key.get_value::<u32, _>("BlockTelemetry") {
            policy.block_telemetry = Some(v != 0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Settings;

    #[test]
    fn empty_policy_doesnt_touch_settings() {
        let mut s = Settings::default();
        let original_channel = s.advanced.update_channel.clone();
        let original_auto = s.general.check_updates_automatically;
        Policy::default().apply_to(&mut s);
        assert_eq!(s.advanced.update_channel, original_channel);
        assert_eq!(s.general.check_updates_automatically, original_auto);
    }

    #[test]
    fn policy_overrides_update_channel() {
        let mut s = Settings::default();
        s.advanced.update_channel = "stable".into();
        let p = Policy {
            update_channel: Some("beta".into()),
            ..Default::default()
        };
        p.apply_to(&mut s);
        assert_eq!(s.advanced.update_channel, "beta");
    }

    #[test]
    fn invalid_channel_value_is_ignored() {
        let mut s = Settings::default();
        s.advanced.update_channel = "stable".into();
        let p = Policy {
            update_channel: Some("stabel-typo".into()),
            ..Default::default()
        };
        p.apply_to(&mut s);
        // Untouched: typo'd policy values fall back to user setting.
        assert_eq!(s.advanced.update_channel, "stable");
    }

    #[test]
    fn policy_overrides_auto_update() {
        let mut s = Settings::default();
        s.general.check_updates_automatically = true;
        let p = Policy {
            check_updates_automatically: Some(false),
            ..Default::default()
        };
        p.apply_to(&mut s);
        assert!(!s.general.check_updates_automatically);
    }

    #[test]
    fn block_model_downloads_helper() {
        assert!(!Policy::default().model_downloads_blocked());
        let p = Policy {
            block_model_downloads: Some(true),
            ..Default::default()
        };
        assert!(p.model_downloads_blocked());
        let p = Policy {
            block_model_downloads: Some(false),
            ..Default::default()
        };
        assert!(!p.model_downloads_blocked());
    }

    #[test]
    fn is_active_reports_correctly() {
        assert!(!Policy::default().is_active());
        assert!(Policy {
            update_channel: Some("beta".into()),
            ..Default::default()
        }
        .is_active());
        assert!(Policy {
            check_updates_automatically: Some(true),
            ..Default::default()
        }
        .is_active());
        assert!(Policy {
            block_system_audio: Some(true),
            ..Default::default()
        }
        .is_active());
    }

    #[test]
    fn block_system_audio_forces_microphone() {
        use crate::domain::settings::CaptureSource;
        let mut s = Settings::default();
        s.audio.capture_source = CaptureSource::System;
        let p = Policy {
            block_system_audio: Some(true),
            ..Default::default()
        };
        p.apply_to(&mut s);
        assert_eq!(s.audio.capture_source, CaptureSource::Microphone);

        // Mixed → Microphone too.
        s.audio.capture_source = CaptureSource::Mixed;
        p.apply_to(&mut s);
        assert_eq!(s.audio.capture_source, CaptureSource::Microphone);
    }

    #[test]
    fn block_system_audio_false_doesnt_touch_user_choice() {
        use crate::domain::settings::CaptureSource;
        let mut s = Settings::default();
        s.audio.capture_source = CaptureSource::System;
        // Explicit `false` (admin opted out) is NOT the same as `None`
        // (no policy set) — but neither should override the user's
        // System pick. apply_to only forces when the policy is `Some(true)`.
        let p = Policy {
            block_system_audio: Some(false),
            ..Default::default()
        };
        p.apply_to(&mut s);
        assert_eq!(s.audio.capture_source, CaptureSource::System);
    }

    #[test]
    fn history_blocked_helper() {
        assert!(!Policy::default().history_blocked());
        assert!(Policy {
            block_history: Some(true),
            ..Default::default()
        }
        .history_blocked());
        assert!(!Policy {
            block_history: Some(false),
            ..Default::default()
        }
        .history_blocked());
    }

    /// `history_export_blocked()` is true when either flag is set.
    /// `BlockHistory` implies `BlockHistoryExport` because there is
    /// nothing fresh to export — the strict-superset relationship is
    /// part of the documented semantics, not just incidental.
    #[test]
    fn history_export_blocked_respects_both_flags() {
        assert!(!Policy::default().history_export_blocked());
        assert!(Policy {
            block_history_export: Some(true),
            ..Default::default()
        }
        .history_export_blocked());
        // BlockHistory alone is enough — superset semantics.
        assert!(Policy {
            block_history: Some(true),
            ..Default::default()
        }
        .history_export_blocked());
        // Both set is still blocked.
        assert!(Policy {
            block_history: Some(true),
            block_history_export: Some(true),
            ..Default::default()
        }
        .history_export_blocked());
        // Both explicitly false → not blocked.
        assert!(!Policy {
            block_history: Some(false),
            block_history_export: Some(false),
            ..Default::default()
        }
        .history_export_blocked());
    }

    #[test]
    fn telemetry_blocked_helper() {
        assert!(!Policy::default().telemetry_blocked());
        assert!(Policy {
            block_telemetry: Some(true),
            ..Default::default()
        }
        .telemetry_blocked());
        assert!(!Policy {
            block_telemetry: Some(false),
            ..Default::default()
        }
        .telemetry_blocked());
    }

    /// New keys flip `is_active()` independently of the existing set.
    /// Catches regressions where a new pinnable value was added to
    /// the struct but not to the active-set predicate.
    #[test]
    fn is_active_includes_new_keys() {
        assert!(Policy {
            block_history: Some(true),
            ..Default::default()
        }
        .is_active());
        assert!(Policy {
            block_history_export: Some(true),
            ..Default::default()
        }
        .is_active());
        assert!(Policy {
            block_telemetry: Some(true),
            ..Default::default()
        }
        .is_active());
    }
}
