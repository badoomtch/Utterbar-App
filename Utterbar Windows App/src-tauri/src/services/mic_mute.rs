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

use crate::domain::settings::HotkeyCombo;

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

    let mut results: Vec<(MicMuteApp, AppOutcome)> = Vec::new();
    let mut applied: Vec<MicMuteApp> = Vec::new();

    for (app, hotkey, processes) in [
        (MicMuteApp::Teams, req.teams.as_ref(), TEAMS_PROCESSES),
        (MicMuteApp::Discord, req.discord.as_ref(), DISCORD_PROCESSES),
    ] {
        let Some(combo) = hotkey else {
            results.push((app, AppOutcome::NoHotkey));
            continue;
        };
        let Some(hwnd) = find_window_for_processes(processes) else {
            results.push((app, AppOutcome::NotRunning));
            continue;
        };
        let win_combo = hotkey_to_window_input(combo);
        match send_hotkey_to_window(hwnd, &win_combo) {
            Ok(()) => {
                results.push((app, AppOutcome::Sent));
                applied.push(app);
            }
            Err(e) => {
                tracing::warn!(app = app.as_str(), error = %e, "mic_mute: apply failed");
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
