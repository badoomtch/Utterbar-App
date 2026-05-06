//! Experimental Teams + Discord mic-mute persistence + lifecycle helpers.
//!
//! This module contains the *always-compiled* domain logic for the
//! mic-mute feature: the on-disk state file (`mic_mute_state.json`),
//! the apply / restore orchestration shape, and the dictation-lifecycle
//! integration points.
//!
//! The actual `SendInput` keystroke delivery lives behind the
//! `experimental_mic_mute` Cargo feature in
//! `platform::windows::window_send_input`. The two halves are wired
//! together in this module via [`MicMuteService::apply`] and
//! [`MicMuteService::restore`], whose bodies are feature-gated. In
//! default builds, the service exists as a no-op so the lifecycle hooks
//! in `commands/dictation.rs` can call it unconditionally without
//! `cfg(feature)` sprinkled around the recording path.
//!
//! See `docs/decisions/true-per-app-microphone-mute-v1-spec.md` for
//! the product spec.
//!
//! Branch 3a (this commit) lands the domain types, persistence, and
//! the apply / restore service surface. Branches 3b (Settings UI +
//! Tauri commands) and 3c (banner + dictation lifecycle hook) wire
//! these items into the rest of the codebase. Until then, several
//! items here are intentionally unreferenced; allow dead-code
//! warnings until Branch 3c lands.

#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use ulid::Ulid;

use tauri::{AppHandle, Emitter};

use crate::domain::events::{MicMutePhase, MicMuteStatus, MicMuteStatusApp, EVT_MIC_MUTE_STATUS};
use crate::domain::settings::{HotkeyCombo, MicMuteSettings};

/// Filename of the on-disk state file under `%APPDATA%\Utterbar\`.
/// Public so tests + diagnostics tooling can reference the same constant.
pub const STATE_FILENAME: &str = "mic_mute_state.json";

/// Stale-state cutoff. If the on-disk state file is older than this when
/// Utterbar launches, it's deleted-and-ignored rather than triggering a
/// restore. The whole feature is for catching short-window crashes; a
/// day-old file is something the user has long since dealt with manually.
pub const STALE_AFTER_HOURS: u64 = 24;

/// Closed enum of supported apps. Adding a value (e.g. `Zoom`,
/// `Webex`) is a code change; the field on disk is a string for
/// forward-compat with future v2 desktops reading a v1 file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MicMuteApp {
    Teams,
    Discord,
}

impl MicMuteApp {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Teams => "teams",
            Self::Discord => "discord",
        }
    }
}

/// What we believed the app's mute state was at apply time. v1 always
/// writes `Unknown` because the prototype does not read app state.
/// Reserved for a future v2 that uses UIA to read mute-button state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PreviousMuteState {
    #[default]
    Unknown,
    Muted,
    Unmuted,
}

/// One per-app entry inside the on-disk state file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicMuteAppEntry {
    pub app: MicMuteApp,
    #[serde(default)]
    pub previous_mute_state: PreviousMuteState,
}

/// On-disk shape of `mic_mute_state.json`. Schema is `1` for v1; a
/// schema-version bump deletes-and-ignores older files rather than
/// migrating them, since the file is ephemeral state about an
/// in-flight dictation, not durable user data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicMuteState {
    pub schema: u32,
    pub session_id: String,
    /// ISO 8601 UTC string with seconds precision.
    pub applied_at_utc: String,
    pub apps: Vec<MicMuteAppEntry>,
}

impl MicMuteState {
    pub fn new(apps: Vec<MicMuteApp>) -> Self {
        Self {
            schema: 1,
            session_id: Ulid::new().to_string(),
            applied_at_utc: now_utc_iso8601(),
            apps: apps
                .into_iter()
                .map(|app| MicMuteAppEntry {
                    app,
                    previous_mute_state: PreviousMuteState::Unknown,
                })
                .collect(),
        }
    }
}

fn now_utc_iso8601() -> String {
    // Avoid pulling chrono just for this one call; the existing `time`
    // crate is already a dep and produces an ISO-8601 string for free.
    let now = time::OffsetDateTime::now_utc();
    now.format(&time::format_description::well_known::Iso8601::DEFAULT)
        .unwrap_or_else(|_| String::from("1970-01-01T00:00:00Z"))
}

/// Resolve the path to `mic_mute_state.json`. Returns an error if the
/// AppData directory cannot be located (which shouldn't happen on a
/// running Utterbar but we propagate cleanly rather than panic).
pub fn state_path() -> std::io::Result<PathBuf> {
    let dir = crate::util::paths::data_dir().map_err(|e| std::io::Error::other(e.to_string()))?;
    Ok(dir.join(STATE_FILENAME))
}

/// Write the state file. Best-effort: any IO error is logged and
/// returned so the caller can decide whether to surface it (the caller
/// in v1 does not — a state-write failure is logged and we proceed
/// because the alternative is refusing to mute when the user asked us
/// to, which is worse than being unable to recover from a crash).
pub fn write_state(state: &MicMuteState) -> std::io::Result<()> {
    let path = state_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_vec_pretty(state)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    fs::write(&path, json)?;
    Ok(())
}

/// Read the state file. Returns `Ok(None)` if it does not exist or is
/// malformed (we treat malformed as no-state rather than propagating
/// the parse error: a bad file from a long-ago version should not
/// block the user). Returns `Ok(Some(_))` on a clean read.
pub fn read_state() -> std::io::Result<Option<MicMuteState>> {
    let path = state_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let bytes = match fs::read(&path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };
    Ok(serde_json::from_slice::<MicMuteState>(&bytes).ok())
}

/// Delete the state file. Idempotent: missing-file is success.
pub fn delete_state() -> std::io::Result<()> {
    let path = state_path()?;
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

/// Returns true if the file's `applied_at_utc` is older than the stale
/// cutoff. A `None` (couldn't parse the timestamp) is also treated as
/// stale, since we can't reason about it.
pub fn is_stale(state: &MicMuteState) -> bool {
    let Ok(parsed) = time::OffsetDateTime::parse(
        &state.applied_at_utc,
        &time::format_description::well_known::Iso8601::DEFAULT,
    ) else {
        return true;
    };
    let age = SystemTime::now()
        .duration_since(SystemTime::from(parsed))
        .unwrap_or_default();
    age.as_secs() > STALE_AFTER_HOURS * 3600
}

/// Outcome of attempting to apply mic-mute to a single app.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppOutcome {
    /// Hotkey was sent and `SendInput` returned a non-zero count.
    Sent,
    /// Target app's window was not found (not running, or a variant we
    /// don't recognise). We did not fail; we just did nothing.
    NotRunning,
    /// `SendInput` returned an error or count mismatch. The caller
    /// surfaces this as a restore-failure banner.
    SendFailed,
    /// The app's hotkey is not configured (`hotkey: None` in settings).
    /// Not an error: the toggle is on but the user hasn't told us what
    /// the call app's mute key is yet, so we skip.
    NoHotkey,
}

/// Banned-fields snapshot test. Asserts that a serialised state file
/// contains nothing that could leak a transcript, window title,
/// clipboard fragment, vocabulary entry, licence key, or any other
/// secret listed in the parent decision doc.
#[cfg(test)]
mod banned_fields_test {
    use super::*;

    #[test]
    fn serialised_state_contains_no_banned_fields() {
        let state = MicMuteState::new(vec![MicMuteApp::Teams, MicMuteApp::Discord]);
        let json = serde_json::to_string(&state).unwrap();

        // Every banned-fields term from
        // docs/decisions/true-per-app-microphone-mute.md "Banned fields".
        // Each match is a hard fail.
        let banned = [
            "transcript",
            "window_title",
            "active_window",
            "clipboard",
            "vocabulary",
            "licence_key",
            "license_key",
            "mac_address",
            "machine_fingerprint",
            "device_id",
            "email",
            "ip_address",
            "panic",
            "stack_trace",
            "selection",
            "focused_app",
            "hostname",
        ];
        for term in banned {
            assert!(
                !json.contains(term),
                "serialised state contains banned field/term `{}`: {}",
                term,
                json
            );
        }

        // Also assert that the literal hotkey combo (which IS in the
        // settings struct, not the state file) does not leak in here.
        // The state file should only carry app id + previous_mute_state
        // + ULID + timestamp.
        assert!(!json.contains("hotkey"));
        assert!(!json.contains("vk"));
        assert!(!json.contains("ctrl"));
        assert!(!json.contains("modifier"));
    }
}

#[cfg(test)]
mod state_roundtrip_test {
    use super::*;

    #[test]
    fn fresh_state_round_trips_through_json() {
        let state = MicMuteState::new(vec![MicMuteApp::Teams, MicMuteApp::Discord]);
        let json = serde_json::to_string(&state).unwrap();
        let back: MicMuteState = serde_json::from_str(&json).unwrap();
        assert_eq!(back.schema, 1);
        assert_eq!(back.apps.len(), 2);
        assert_eq!(back.apps[0].app, MicMuteApp::Teams);
        assert_eq!(back.apps[1].app, MicMuteApp::Discord);
        assert_eq!(back.apps[0].previous_mute_state, PreviousMuteState::Unknown);
    }

    #[test]
    fn schema_version_is_one() {
        let state = MicMuteState::new(vec![MicMuteApp::Teams]);
        assert_eq!(state.schema, 1);
    }

    #[test]
    fn ulid_session_ids_are_unique() {
        let a = MicMuteState::new(vec![]);
        let b = MicMuteState::new(vec![]);
        assert_ne!(a.session_id, b.session_id);
        // ULID is 26 chars in canonical encoding
        assert_eq!(a.session_id.len(), 26);
        assert_eq!(b.session_id.len(), 26);
    }

    #[test]
    fn apps_field_serialises_lowercase() {
        let state = MicMuteState::new(vec![MicMuteApp::Teams, MicMuteApp::Discord]);
        let json = serde_json::to_string(&state).unwrap();
        // "teams" and "discord" lowercased per #[serde(rename_all = "lowercase")]
        assert!(json.contains("\"teams\""));
        assert!(json.contains("\"discord\""));
        // Capitalised forms must NOT appear.
        assert!(!json.contains("\"Teams\""));
        assert!(!json.contains("\"Discord\""));
    }
}

// =============================================================================
// Live apply / restore path. Behind the experimental_mic_mute feature
// because it depends on `platform::windows::window_send_input`.
// =============================================================================

/// Inputs to the apply-on-start hook. Built by the lifecycle hook in
/// `commands::dictation::start_recording_inner` from the user's
/// settings; passed through to the live apply path so the lifecycle
/// caller does not have to know about `platform::windows::*`.
#[derive(Debug, Clone)]
pub struct MicMuteApplyRequest {
    pub teams: Option<HotkeyCombo>,
    pub discord: Option<HotkeyCombo>,
}

impl MicMuteApplyRequest {
    /// Returns the list of apps that have a hotkey configured (so
    /// `SendInput` has something to send). Apps with `None` are
    /// skipped silently.
    pub fn configured_apps(&self) -> Vec<MicMuteApp> {
        let mut out = Vec::with_capacity(2);
        if self.teams.is_some() {
            out.push(MicMuteApp::Teams);
        }
        if self.discord.is_some() {
            out.push(MicMuteApp::Discord);
        }
        out
    }
}

/// Inputs to the restore-on-stop hook.
#[derive(Debug, Clone)]
pub struct MicMuteRestoreRequest {
    pub teams: Option<HotkeyCombo>,
    pub discord: Option<HotkeyCombo>,
    /// Which apps had mute applied at start time. We only restore the
    /// ones that were applied; an app that wasn't running at start
    /// time gets no toggle keystroke at stop time even if it has come
    /// online since.
    pub applied: Vec<MicMuteApp>,
}

/// Apply mic-mute to the requested apps. In default builds, this is a
/// no-op stub returning `NotRunning` for every configured app, which
/// is the correct semantics: without the experimental_mic_mute feature
/// flipped on, no `SendInput` happens, and the state file is not
/// written. The lifecycle hook can call this unconditionally.
#[cfg(not(feature = "experimental_mic_mute"))]
pub fn apply(_req: &MicMuteApplyRequest) -> Vec<(MicMuteApp, AppOutcome)> {
    // Feature-off: pretend nothing matched. The state file is NOT
    // written because no apps were actually muted.
    Vec::new()
}

/// Apply mic-mute to the requested apps. Feature-on path: looks up
/// each app's window by process name, sends the configured hotkey,
/// records per-app outcome. Writes the state file if at least one app
/// was successfully sent (so we can restore on crash).
///
/// Logs failures locally; never logs the user's actual hotkey combo
/// (only `app=teams` / `app=discord` and the outcome).
#[cfg(feature = "experimental_mic_mute")]
pub fn apply(req: &MicMuteApplyRequest) -> Vec<(MicMuteApp, AppOutcome)> {
    use crate::platform::windows::window_send_input::{
        find_window_for_processes, send_hotkey_to_window, DISCORD_PROCESSES, TEAMS_PROCESSES,
    };

    tracing::info!("mic_mute diag: feature-on apply() entered");

    let mut results: Vec<(MicMuteApp, AppOutcome)> = Vec::new();
    let mut applied: Vec<MicMuteApp> = Vec::new();

    for (app, hotkey, processes) in [
        (MicMuteApp::Teams, req.teams.as_ref(), TEAMS_PROCESSES),
        (MicMuteApp::Discord, req.discord.as_ref(), DISCORD_PROCESSES),
    ] {
        let Some(_combo) = hotkey else {
            tracing::info!(
                app = app.as_str(),
                "mic_mute diag: NoHotkey, hotkey is None for this app"
            );
            results.push((app, AppOutcome::NoHotkey));
            continue;
        };
        let Some(hwnd) = find_window_for_processes(processes) else {
            tracing::info!(
                app = app.as_str(),
                processes = ?processes,
                "mic_mute diag: NotRunning, find_window_for_processes returned None"
            );
            results.push((app, AppOutcome::NotRunning));
            continue;
        };
        tracing::info!(
            app = app.as_str(),
            hwnd = ?hwnd,
            "mic_mute diag: window found, sending hotkey"
        );
        let win_combo = hotkey_to_window_input(_combo);
        match send_hotkey_to_window(hwnd, &win_combo) {
            Ok(()) => {
                tracing::info!(
                    app = app.as_str(),
                    "mic_mute diag: send_hotkey_to_window returned Ok"
                );
                results.push((app, AppOutcome::Sent));
                applied.push(app);
            }
            Err(e) => {
                tracing::warn!(
                    app = app.as_str(),
                    error = %e,
                    "mic_mute diag: send_hotkey_to_window returned Err"
                );
                results.push((app, AppOutcome::SendFailed));
            }
        }
    }

    if !applied.is_empty() {
        let state = MicMuteState::new(applied);
        if let Err(e) = write_state(&state) {
            tracing::warn!(error = %e, "mic_mute: state-file write failed");
        }
    }

    results
}

#[cfg(feature = "experimental_mic_mute")]
fn hotkey_to_window_input(
    combo: &HotkeyCombo,
) -> crate::platform::windows::window_send_input::Hotkey {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        VIRTUAL_KEY, VK_CONTROL, VK_LWIN, VK_MENU, VK_SHIFT,
    };
    let mut mods: Vec<VIRTUAL_KEY> = Vec::with_capacity(4);
    if combo.ctrl {
        mods.push(VK_CONTROL);
    }
    if combo.shift {
        mods.push(VK_SHIFT);
    }
    if combo.alt {
        mods.push(VK_MENU);
    }
    if combo.win {
        mods.push(VK_LWIN);
    }
    crate::platform::windows::window_send_input::Hotkey {
        modifiers: mods,
        main: VIRTUAL_KEY(combo.vk),
    }
}

/// Restore mic-mute by sending the toggle keystroke a second time to
/// each previously-applied app. Default builds: no-op; the state file
/// was never written, so there's nothing to restore.
#[cfg(not(feature = "experimental_mic_mute"))]
pub fn restore(_req: &MicMuteRestoreRequest) -> Vec<(MicMuteApp, AppOutcome)> {
    Vec::new()
}

/// Restore mic-mute by sending the toggle keystroke a second time.
/// Removes each successfully-restored app from the state file; deletes
/// the file when the apps array is empty.
#[cfg(feature = "experimental_mic_mute")]
pub fn restore(req: &MicMuteRestoreRequest) -> Vec<(MicMuteApp, AppOutcome)> {
    use crate::platform::windows::window_send_input::{
        find_window_for_processes, send_hotkey_to_window, DISCORD_PROCESSES, TEAMS_PROCESSES,
    };

    let mut results: Vec<(MicMuteApp, AppOutcome)> = Vec::new();
    let mut still_applied: Vec<MicMuteApp> = Vec::new();

    for app in &req.applied {
        let (hotkey, processes) = match app {
            MicMuteApp::Teams => (req.teams.as_ref(), TEAMS_PROCESSES),
            MicMuteApp::Discord => (req.discord.as_ref(), DISCORD_PROCESSES),
        };
        let Some(combo) = hotkey else {
            // No hotkey now? The user must have cleared it mid-dictation.
            // We can't restore; record as failure so the banner fires.
            results.push((*app, AppOutcome::NoHotkey));
            still_applied.push(*app);
            continue;
        };
        let Some(hwnd) = find_window_for_processes(processes) else {
            // App went away; the user is presumably no longer in the
            // call. Treat as success (nothing to restore).
            results.push((*app, AppOutcome::NotRunning));
            continue;
        };
        let win_combo = hotkey_to_window_input(combo);
        match send_hotkey_to_window(hwnd, &win_combo) {
            Ok(()) => {
                results.push((*app, AppOutcome::Sent));
            }
            Err(e) => {
                tracing::warn!(app = app.as_str(), error = %e, "mic_mute: restore failed");
                results.push((*app, AppOutcome::SendFailed));
                still_applied.push(*app);
            }
        }
    }

    // Update the state file: keep only the apps that still need
    // restoring; if none, delete it.
    if still_applied.is_empty() {
        if let Err(e) = delete_state() {
            tracing::warn!(error = %e, "mic_mute: state-file delete failed");
        }
    } else {
        let state = MicMuteState::new(still_applied);
        if let Err(e) = write_state(&state) {
            tracing::warn!(error = %e, "mic_mute: state-file rewrite failed");
        }
    }

    results
}

/// Restore-on-launch hook: read any stale state file from disk, attempt
/// restore, surface the banner regardless of keystroke result. Called
/// once during app boot.
///
/// In default builds this still runs and reads the file (so a beta
/// user who flips back to stable doesn't accumulate stale state files
/// indefinitely); but the `restore` call is a no-op stub, and the
/// file is deleted unconditionally. In feature-on builds, the live
/// `restore` path runs.
pub fn restore_on_launch(req_lookup: impl FnOnce(&[MicMuteApp]) -> MicMuteRestoreRequest) {
    let state = match read_state() {
        Ok(Some(s)) => s,
        Ok(None) => return,
        Err(e) => {
            tracing::warn!(error = %e, "mic_mute: state-file read failed on launch");
            return;
        }
    };

    if is_stale(&state) {
        tracing::info!(
            "mic_mute: state file older than {}h — deleting and ignoring",
            STALE_AFTER_HOURS
        );
        let _ = delete_state();
        return;
    }

    let apps: Vec<MicMuteApp> = state.apps.iter().map(|e| e.app).collect();
    if apps.is_empty() {
        let _ = delete_state();
        return;
    }

    let req = req_lookup(&apps);
    let results = restore(&req);

    tracing::info!(
        outcomes = ?results,
        "mic_mute: restore-on-launch attempted"
    );
    // The banner emit is the caller's job (it has the Tauri AppHandle).
    // This function returns; the caller observes results via the
    // tracing log line and the on-disk file state.
}

// =============================================================================
// High-level orchestrators called from the dictation lifecycle.
// =============================================================================

/// Build an apply request from the user's settings. Returns `None` if
/// neither toggle is enabled, in which case the caller skips the
/// whole apply path (no log lines, no event, no state-file write).
pub fn apply_request_from_settings(settings: &MicMuteSettings) -> Option<MicMuteApplyRequest> {
    let teams = if settings.teams.enabled {
        settings.teams.hotkey.clone()
    } else {
        None
    };
    let discord = if settings.discord.enabled {
        settings.discord.hotkey.clone()
    } else {
        None
    };
    if teams.is_none() && discord.is_none() {
        return None;
    }
    Some(MicMuteApplyRequest { teams, discord })
}

/// Build a restore request. Distinct from apply because the `applied`
/// list comes from the active recording context, not from "what's
/// currently enabled in Settings": a user might have flipped a toggle
/// off mid-recording and we still need to restore whatever was
/// actually applied at start time.
pub fn restore_request_from_settings(
    settings: &MicMuteSettings,
    applied: Vec<MicMuteApp>,
) -> MicMuteRestoreRequest {
    MicMuteRestoreRequest {
        teams: settings.teams.hotkey.clone(),
        discord: settings.discord.hotkey.clone(),
        applied,
    }
}

/// Display label for an app, used in user-facing banner copy. Closed
/// match so adding an app variant requires updating this and the
/// React banner together.
pub fn app_label(app: MicMuteApp) -> &'static str {
    match app {
        MicMuteApp::Teams => "Teams",
        MicMuteApp::Discord => "Discord",
    }
}

/// Compose the user-facing banner copy for the "we just attempted to
/// mute these apps at recording start" phase. Pinned by tests below.
pub fn compose_recording_started_message(apps: &[MicMuteApp]) -> String {
    match apps.len() {
        0 => String::new(),
        1 => format!(
            "Tried to mute {} while recording. Confirm in {}.",
            app_label(apps[0]),
            app_label(apps[0])
        ),
        _ => {
            let names: Vec<&str> = apps.iter().map(|a| app_label(*a)).collect();
            format!(
                "Tried to mute {} while recording. Confirm in those apps.",
                names.join(" and ")
            )
        }
    }
}

/// Compose the user-facing banner copy for the "restore succeeded"
/// phase. Auto-dismisses after 5 seconds in the React layer.
pub fn compose_restore_succeeded_message(apps: &[MicMuteApp]) -> String {
    match apps.len() {
        0 => String::new(),
        1 => format!(
            "Tried to unmute {}. Please confirm in {}.",
            app_label(apps[0]),
            app_label(apps[0])
        ),
        _ => {
            let names: Vec<&str> = apps.iter().map(|a| app_label(*a)).collect();
            format!(
                "Tried to unmute {}. Please confirm in both.",
                names.join(" and ")
            )
        }
    }
}

/// Compose the sticky failure banner copy. Apps listed are the ones
/// that failed restore; the success-set is implicit on the others.
pub fn compose_restore_failed_message(apps: &[MicMuteApp]) -> String {
    match apps.len() {
        0 => String::new(),
        1 => format!(
            "Utterbar may have left you muted in {}. Open {} and check the mic icon.",
            app_label(apps[0]),
            app_label(apps[0])
        ),
        _ => {
            let names: Vec<&str> = apps.iter().map(|a| app_label(*a)).collect();
            format!(
                "Utterbar may have left you muted in {}. Open both apps and check the mic icons.",
                names.join(" and ")
            )
        }
    }
}

fn outcome_to_string(o: AppOutcome) -> &'static str {
    match o {
        AppOutcome::Sent => "sent",
        AppOutcome::NotRunning => "not_running",
        AppOutcome::SendFailed => "send_failed",
        AppOutcome::NoHotkey => "no_hotkey",
    }
}

/// Build the per-app payload entries for a `MicMuteStatus` event,
/// stamping each with its outcome.
fn status_apps_from_outcomes(outcomes: &[(MicMuteApp, AppOutcome)]) -> Vec<MicMuteStatusApp> {
    outcomes
        .iter()
        .map(|(app, out)| MicMuteStatusApp {
            app: app.as_str().to_string(),
            label: app_label(*app).to_string(),
            outcome: Some(outcome_to_string(*out).to_string()),
        })
        .collect()
}

fn status_apps_no_outcome(apps: &[MicMuteApp]) -> Vec<MicMuteStatusApp> {
    apps.iter()
        .map(|app| MicMuteStatusApp {
            app: app.as_str().to_string(),
            label: app_label(*app).to_string(),
            outcome: None,
        })
        .collect()
}

/// Apply at recording start, log per-app outcome, emit a banner
/// event, return the list of apps that were successfully sent (so
/// the lifecycle caller can store it on the recording context).
pub fn apply_with_event(handle: &AppHandle, req: &MicMuteApplyRequest) -> Vec<MicMuteApp> {
    let outcomes = apply(req);
    let applied: Vec<MicMuteApp> = outcomes
        .iter()
        .filter(|(_, o)| matches!(o, AppOutcome::Sent))
        .map(|(a, _)| *a)
        .collect();
    if applied.is_empty() {
        // Nothing landed: don't show a banner. The user might have
        // both toggles on but neither call app running, which is fine.
        for (app, out) in &outcomes {
            tracing::info!(
                app = app.as_str(),
                outcome = outcome_to_string(*out),
                "mic_mute: apply skipped (app not running or no hotkey)"
            );
        }
        return applied;
    }
    let message = compose_recording_started_message(&applied);
    let payload = MicMuteStatus {
        phase: MicMutePhase::RecordingStarted,
        apps: status_apps_from_outcomes(&outcomes),
        message,
    };
    let _ = handle.emit(EVT_MIC_MUTE_STATUS, &payload);
    applied
}

/// Restore at recording stop / cancel / rollback, log per-app
/// outcome, emit success or failure banner event. Caller passes the
/// list of apps that were successfully sent at start time.
pub fn restore_with_event(handle: &AppHandle, req: &MicMuteRestoreRequest) {
    if req.applied.is_empty() {
        return;
    }
    let outcomes = restore(req);

    // Sent + NotRunning are both "success" from the user's POV: app
    // gone means there's nothing to mute; sent means we asked for the
    // unmute and `SendInput` returned ok. SendFailed and NoHotkey are
    // failures.
    let failures: Vec<MicMuteApp> = outcomes
        .iter()
        .filter(|(_, o)| matches!(o, AppOutcome::SendFailed | AppOutcome::NoHotkey))
        .map(|(a, _)| *a)
        .collect();
    let successes: Vec<MicMuteApp> = outcomes
        .iter()
        .filter(|(_, o)| matches!(o, AppOutcome::Sent | AppOutcome::NotRunning))
        .map(|(a, _)| *a)
        .collect();

    if failures.is_empty() {
        let message = compose_restore_succeeded_message(&successes);
        let payload = MicMuteStatus {
            phase: MicMutePhase::RestoreSucceeded,
            apps: status_apps_from_outcomes(&outcomes),
            message,
        };
        let _ = handle.emit(EVT_MIC_MUTE_STATUS, &payload);
    } else {
        let message = compose_restore_failed_message(&failures);
        let payload = MicMuteStatus {
            phase: MicMutePhase::RestoreFailed,
            apps: status_apps_from_outcomes(&outcomes),
            message,
        };
        let _ = handle.emit(EVT_MIC_MUTE_STATUS, &payload);
    }
}

/// Restore-on-launch wrapper: if state file exists and has entries,
/// attempt restore using the user's current settings (which is the
/// only source of truth we have for hotkeys; the state file
/// deliberately doesn't carry them). Emits a sticky-failure banner if
/// any restore failed OR if the state file existed at all (the user
/// has been muted unknowingly for some interval and deserves to be
/// told regardless). Quiet success: no event when the state file was
/// absent.
pub fn restore_on_launch_with_event(handle: &AppHandle, settings: &MicMuteSettings) {
    let state = match read_state() {
        Ok(Some(s)) => s,
        Ok(None) => return,
        Err(e) => {
            tracing::warn!(error = %e, "mic_mute: state-file read failed on launch");
            return;
        }
    };

    if is_stale(&state) {
        tracing::info!(
            "mic_mute: state file older than {}h on launch, deleting and ignoring",
            STALE_AFTER_HOURS
        );
        let _ = delete_state();
        return;
    }

    let apps: Vec<MicMuteApp> = state.apps.iter().map(|e| e.app).collect();
    if apps.is_empty() {
        let _ = delete_state();
        return;
    }

    let req = MicMuteRestoreRequest {
        teams: settings.teams.hotkey.clone(),
        discord: settings.discord.hotkey.clone(),
        applied: apps.clone(),
    };
    let outcomes = restore(&req);

    // Always emit the failure-grade banner on launch, even if every
    // restore returned ok. The user has been muted unknowingly for
    // some interval; they should look at the call apps regardless.
    let payload = MicMuteStatus {
        phase: MicMutePhase::RestoreOnLaunchFailed,
        apps: if outcomes.is_empty() {
            status_apps_no_outcome(&apps)
        } else {
            status_apps_from_outcomes(&outcomes)
        },
        message: compose_restore_failed_message(&apps),
    };
    let _ = handle.emit(EVT_MIC_MUTE_STATUS, &payload);
}

#[cfg(test)]
mod compose_message_tests {
    use super::*;

    #[test]
    fn recording_started_one_app() {
        assert_eq!(
            compose_recording_started_message(&[MicMuteApp::Teams]),
            "Tried to mute Teams while recording. Confirm in Teams."
        );
    }

    #[test]
    fn recording_started_both_apps() {
        assert_eq!(
            compose_recording_started_message(&[MicMuteApp::Teams, MicMuteApp::Discord]),
            "Tried to mute Teams and Discord while recording. Confirm in those apps."
        );
    }

    #[test]
    fn recording_started_empty_yields_empty_string() {
        assert_eq!(compose_recording_started_message(&[]), "");
    }

    #[test]
    fn restore_succeeded_one_app() {
        assert_eq!(
            compose_restore_succeeded_message(&[MicMuteApp::Discord]),
            "Tried to unmute Discord. Please confirm in Discord."
        );
    }

    #[test]
    fn restore_succeeded_both_apps() {
        assert_eq!(
            compose_restore_succeeded_message(&[MicMuteApp::Teams, MicMuteApp::Discord]),
            "Tried to unmute Teams and Discord. Please confirm in both."
        );
    }

    #[test]
    fn restore_failed_one_app() {
        assert_eq!(
            compose_restore_failed_message(&[MicMuteApp::Teams]),
            "Utterbar may have left you muted in Teams. Open Teams and check the mic icon."
        );
    }

    #[test]
    fn restore_failed_both_apps() {
        assert_eq!(
            compose_restore_failed_message(&[MicMuteApp::Teams, MicMuteApp::Discord]),
            "Utterbar may have left you muted in Teams and Discord. Open both apps and check the mic icons."
        );
    }
}

/// Source-grep guard: forbidden symbols that would imply we crossed
/// the v1 design lines. Re-checks at compile time that nothing in this
/// module imports the OS-level capture-endpoint mute API the parent
/// decision doc forbids, and that no UI Automation symbol leaks in
/// (UIA verification is explicitly excluded from v1).
///
/// The check parses the file source and rejects forbidden symbols
/// only outside the test mods (where mention is fine for assertion
/// strings), by stripping the test mods before searching.
#[cfg(test)]
mod source_grep_guard {
    const SOURCE: &str = include_str!("./mic_mute.rs");

    /// Strip every `#[cfg(test)]` mod from SOURCE so symbol mentions
    /// inside test assertions don't trigger the guard against
    /// themselves.
    fn production_source() -> String {
        let mut out = String::with_capacity(SOURCE.len());
        let mut keep = true;
        let mut depth = 0_i32;
        for line in SOURCE.lines() {
            let trimmed = line.trim_start();
            if keep && trimmed.starts_with("#[cfg(test)]") {
                keep = false;
                depth = 0;
                continue;
            }
            if !keep {
                // Track brace balance starting from the next `{`.
                for ch in line.chars() {
                    match ch {
                        '{' => depth += 1,
                        '}' => {
                            depth -= 1;
                            if depth == 0 {
                                keep = true;
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                continue;
            }
            out.push_str(line);
            out.push('\n');
        }
        out
    }

    #[test]
    fn no_capture_endpoint_mute() {
        let src = production_source();
        // Look for the actual import path, not the bare type name.
        assert!(
            !src.contains("Win32_Media_Audio::Endpoints"),
            "mic_mute must not import IAudioEndpointVolume; v1 is hotkey-synthesis only"
        );
        assert!(
            !src.contains(".SetMute("),
            "mic_mute must not call SetMute on any audio session; v1 is hotkey-synthesis only"
        );
    }

    #[test]
    fn no_ui_automation_for_mute_state() {
        let src = production_source();
        assert!(
            !src.contains("uia_thread"),
            "mic_mute must not reach into uia_thread; UIA verification is excluded from v1"
        );
        assert!(
            !src.contains("TogglePattern"),
            "mic_mute must not reference UIA TogglePattern; v1 is honest can-not-verify, not UIA-confidence"
        );
    }
}
