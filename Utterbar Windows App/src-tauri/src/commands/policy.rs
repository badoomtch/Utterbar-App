//! Group-policy snapshot for the frontend.
//!
//! The backend resolves managed values once at boot from
//! `HKLM\SOFTWARE\Policies\Utterbar` (with HKCU as a fallback). The
//! settings UI needs to know which controls are policy-pinned so it
//! can render a "Disabled by your administrator" hint next to them
//! instead of letting the user click a control and have the click
//! silently ignored on the next snapshot. The actual *enforcement*
//! happens server-side (i.e. in the Rust backend) — the UI hint is
//! purely cosmetic.
//!
//! This DTO mirrors the small subset of `Policy` fields the UI needs
//! to render that hint. It is NOT a complete projection of the
//! `Policy` struct: anything the UI doesn't render a hint for stays
//! out, so future internal-only policy values don't leak through to
//! the frontend by default.
//!
//! Restart-to-apply note: the backend reads the registry once at
//! boot. An IT admin who pushes a new policy needs the user to
//! restart Utterbar before the change takes effect. This is the same
//! contract Windows itself has for most policies and is documented
//! on the IT-deployment doc page.
//!
//! Updated 2026-05-01 to add the three privacy controls
//! (`block_history`, `block_history_export`, `block_telemetry`)
//! shipped alongside the managed-licence drop.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::State;
use ts_rs::TS;

use crate::app_state::AppState;
use crate::util::result::CmdResult;

/// Frontend-facing projection of `services::policy::Policy`. Each
/// boolean is `true` if and only if the corresponding registry key
/// is set to a non-zero DWORD; the UI renders the hint and disables
/// the control when the value is true.
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct PolicySnapshot {
    /// True if `UpdateChannel` (REG_SZ) is set to `stable` or `beta`.
    /// Lets the Settings → Updates chip selector render disabled.
    pub update_channel_pinned: bool,
    /// True if `CheckUpdatesAutomatically` (REG_DWORD) is set.
    pub check_updates_pinned: bool,
    /// True if `BlockModelDownloads` is set.
    pub model_downloads_blocked: bool,
    /// True if `BlockSystemAudio` is set. The audio capture-source
    /// picker uses this to disable System / Mixed.
    pub system_audio_blocked: bool,
    /// True if `RequireLicence` is set. The licence panel uses this
    /// to surface a tighter "your IT admin manages this" message
    /// rather than the generic free-tier copy.
    pub licence_required: bool,
    /// True if `BlockHistory` is set. New history rows are not
    /// persisted; the History page renders the existing rows but
    /// the Privacy panel surfaces a disabled hint on the relevant
    /// controls.
    pub history_blocked: bool,
    /// True if `BlockHistoryExport` is set OR `BlockHistory` is set
    /// (the strict-superset semantics — see
    /// `Policy::history_export_blocked` for the rationale). The
    /// export button on the History page renders disabled.
    pub history_export_blocked: bool,
    /// True if `BlockTelemetry` is set. The Dashboard surfaces a
    /// hint that local analytics counters are paused.
    pub telemetry_blocked: bool,
}

/// Read the boot-time policy snapshot. Pure function — no
/// side-effects, no awaits. Called from the settings UI on mount.
#[tauri::command]
pub fn policy_snapshot(state: State<'_, Arc<AppState>>) -> CmdResult<PolicySnapshot> {
    let p = state.settings.policy();
    Ok(PolicySnapshot {
        update_channel_pinned: p.update_channel.is_some(),
        check_updates_pinned: p.check_updates_automatically.is_some(),
        model_downloads_blocked: p.model_downloads_blocked(),
        system_audio_blocked: p.block_system_audio.unwrap_or(false),
        licence_required: p.licence_required(),
        history_blocked: p.history_blocked(),
        history_export_blocked: p.history_export_blocked(),
        telemetry_blocked: p.telemetry_blocked(),
    })
}
