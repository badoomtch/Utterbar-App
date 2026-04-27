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

use crate::domain::Settings;

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
        // `block_model_downloads` is consumed directly by the model
        // service via `Policy::model_downloads_blocked()`; no Settings
        // field to mirror it into.
    }

    /// True if any policy is pinned. Used by the UI to show a small
    /// "managed by your IT admin" hint next to affected controls
    /// rather than letting the user click and have the change silently
    /// ignored.
    pub fn is_active(&self) -> bool {
        self.update_channel.is_some()
            || self.check_updates_automatically.is_some()
            || self.block_model_downloads.is_some()
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
        let p = Policy { block_model_downloads: Some(true), ..Default::default() };
        assert!(p.model_downloads_blocked());
        let p = Policy { block_model_downloads: Some(false), ..Default::default() };
        assert!(!p.model_downloads_blocked());
    }

    #[test]
    fn is_active_reports_correctly() {
        assert!(!Policy::default().is_active());
        assert!(Policy { update_channel: Some("beta".into()), ..Default::default() }.is_active());
        assert!(Policy { check_updates_automatically: Some(true), ..Default::default() }.is_active());
    }
}
