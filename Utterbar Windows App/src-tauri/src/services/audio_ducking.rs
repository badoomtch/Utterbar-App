//! Per-app audio ducking — mutes the **render** (output) volume of specific apps while
//! the user is dictating, and restores it afterwards. Designed for the common scenario
//! where the user is in a Teams / Slack / Discord / Zoom call and wants to dictate into
//! another window without the call audio bleeding into the room AND without Utterbar
//! having to fiddle with the master mic.
//!
//! **Scope (honest).** This mutes OUTPUT audio of the target apps — i.e. you won't hear
//! them while dictating. It does NOT mute the mic feed that those apps are sending into
//! a call. Windows' audio session APIs are output-side; there's no clean way to tell
//! Teams "stop sending my mic to the call" without simulating the app's in-app mute
//! shortcut, which is brittle and app-specific. For now, users who need that should
//! continue to hit the app's own mute button; this feature simply silences the incoming
//! call audio so dictation is easier to hear yourself think over.
//!
//! Windows-only. On other platforms `DuckingService` is a no-op stub so the rest of the
//! codebase can call `.start()` / `.stop()` unconditionally.

use std::sync::Mutex;

pub struct DuckingService {
    // Remembers which (device_id, session_instance_id) pairs we muted, so `restore()`
    // only touches the ones we actually changed. Stored as opaque strings — the Windows
    // side re-enumerates on restore and matches by InstanceIdentifier. Prevents us from
    // unmuting something the user had already manually muted before dictation started.
    muted_sessions: Mutex<Vec<MutedSession>>,
}

#[derive(Clone)]
#[allow(dead_code)] // fields only read on Windows
struct MutedSession {
    instance_id: String,
    exe_basename: String,
}

impl DuckingService {
    pub fn new() -> Self {
        Self {
            muted_sessions: Mutex::new(Vec::new()),
        }
    }

    /// Mute every audio render session whose owning process's EXE basename is in
    /// `target_exes` (case-insensitive). Remembers which ones we actually flipped from
    /// "not muted" to "muted" so `restore()` doesn't clobber a user's pre-existing mute.
    pub fn mute_matching(&self, target_exes: &[String]) {
        #[cfg(windows)]
        {
            match windows_impl::mute_matching(target_exes) {
                Ok(muted) => {
                    if let Ok(mut guard) = self.muted_sessions.lock() {
                        *guard = muted;
                    }
                }
                Err(e) => tracing::warn!(?e, "audio ducking: mute_matching failed"),
            }
        }
        #[cfg(not(windows))]
        {
            let _ = target_exes;
        }
    }

    /// Restore mute state for every session we flipped in the most recent `mute_matching`
    /// call. Safe to call even if `mute_matching` was never called — does nothing.
    pub fn restore(&self) {
        #[cfg(windows)]
        {
            let taken = {
                let Ok(mut guard) = self.muted_sessions.lock() else {
                    return;
                };
                std::mem::take(&mut *guard)
            };
            if taken.is_empty() {
                return;
            }
            if let Err(e) = windows_impl::restore(&taken) {
                tracing::warn!(?e, "audio ducking: restore failed");
            }
        }
    }
}

#[cfg(windows)]
mod windows_impl {
    use super::MutedSession;
    use anyhow::{anyhow, Result};
    use windows::core::Interface;
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::Media::Audio::{
        eConsole, eRender, IAudioSessionControl2, IAudioSessionEnumerator, IAudioSessionManager2,
        IMMDeviceEnumerator, ISimpleAudioVolume, MMDeviceEnumerator,
    };
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_MULTITHREADED,
    };
    use windows::Win32::System::ProcessStatus::GetModuleBaseNameW;
    use windows::Win32::System::Threading::{
        OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_READ,
    };

    /// Small RAII guard that balances CoInitializeEx on this thread. The
    /// tracked field records whether this guard's own CoInitializeEx
    /// actually added to the thread's init count — `S_OK` and `S_FALSE`
    /// both count as initialised-by-us (both require a matching
    /// CoUninitialize), but `RPC_E_CHANGED_MODE` means the thread was
    /// already initialised into a DIFFERENT apartment model and our
    /// call did NOT add to the count. In that case we MUST NOT call
    /// CoUninitialize on Drop — doing so would over-release the count
    /// and destroy COM on this thread for whoever initialised it first.
    ///
    /// Before v0.6.4 this Drop was unconditional, which on tokio
    /// workers that had previously been CoInitialized by the UIA code
    /// path (since moved to a dedicated thread in `uia_thread.rs`)
    /// could over-release and destabilise the ORT / DirectML
    /// allocator state for other tasks — one of the contributing
    /// causes of the STATUS_HEAP_CORRUPTION crashes.
    struct ComGuard {
        initialised_here: bool,
    }
    impl ComGuard {
        fn new() -> Self {
            let hr = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
            // S_OK = apartment entered for the first time on this thread.
            // S_FALSE = already initialised (same mode); counts as our
            //           contribution and must be balanced.
            // Anything else (notably RPC_E_CHANGED_MODE, 0x80010106) =
            // we did NOT add to the count; Drop must NOT CoUninitialize.
            let initialised_here = hr.is_ok(); // is_ok() covers S_OK + S_FALSE
            if !initialised_here {
                tracing::warn!(
                    ?hr,
                    "audio_ducking: CoInitializeEx returned a non-ok HRESULT — skipping CoUninitialize on drop to avoid over-releasing the apartment init count"
                );
            }
            ComGuard { initialised_here }
        }
    }
    impl Drop for ComGuard {
        fn drop(&mut self) {
            if self.initialised_here {
                unsafe { CoUninitialize() };
            }
        }
    }

    pub fn mute_matching(target_exes: &[String]) -> Result<Vec<MutedSession>> {
        if target_exes.is_empty() {
            return Ok(Vec::new());
        }
        let targets_lower: Vec<String> = target_exes.iter().map(|s| s.to_lowercase()).collect();
        let _com = ComGuard::new();

        let manager = unsafe { session_manager()? };
        let enumerator: IAudioSessionEnumerator = unsafe { manager.GetSessionEnumerator()? };
        let count = unsafe { enumerator.GetCount()? };

        let mut muted = Vec::new();
        for i in 0..count {
            let Ok(control) = (unsafe { enumerator.GetSession(i) }) else {
                continue;
            };
            let Ok(ctrl2) = control.cast::<IAudioSessionControl2>() else {
                continue;
            };
            let pid = unsafe { ctrl2.GetProcessId().unwrap_or(0) };
            if pid == 0 {
                continue; // system / inactive
            }
            let Some(exe) = exe_basename_of(pid) else {
                continue;
            };
            let exe_lower = exe.to_lowercase();
            if !targets_lower.iter().any(|t| t == &exe_lower) {
                continue;
            }

            // Only flip sessions that aren't already muted, so restore() won't
            // accidentally unmute something the user muted manually.
            let Ok(vol) = control.cast::<ISimpleAudioVolume>() else {
                continue;
            };
            let already_muted = unsafe { vol.GetMute().map(|b| b.as_bool()).unwrap_or(false) };
            if already_muted {
                continue;
            }

            let Ok(instance_id_pwstr) = (unsafe { ctrl2.GetSessionInstanceIdentifier() }) else {
                continue;
            };
            // PWSTR::to_string is fallible (invalid UTF-16 surrogates) — skip the
            // session if Windows handed us something we can't decode.
            let Ok(instance_id) = (unsafe { instance_id_pwstr.to_string() }) else {
                continue;
            };

            if let Err(e) = unsafe { vol.SetMute(true, std::ptr::null()) } {
                tracing::debug!(?e, exe = %exe, "SetMute(true) failed");
                continue;
            }

            tracing::info!(exe = %exe, pid, "audio ducking: muted output session");
            muted.push(MutedSession {
                instance_id,
                exe_basename: exe,
            });
        }

        Ok(muted)
    }

    pub fn restore(sessions: &[MutedSession]) -> Result<()> {
        if sessions.is_empty() {
            return Ok(());
        }
        let _com = ComGuard::new();
        let manager = unsafe { session_manager()? };
        let enumerator: IAudioSessionEnumerator = unsafe { manager.GetSessionEnumerator()? };
        let count = unsafe { enumerator.GetCount()? };

        for i in 0..count {
            let Ok(control) = (unsafe { enumerator.GetSession(i) }) else {
                continue;
            };
            let Ok(ctrl2) = control.cast::<IAudioSessionControl2>() else {
                continue;
            };
            let Ok(instance_id_pwstr) = (unsafe { ctrl2.GetSessionInstanceIdentifier() }) else {
                continue;
            };
            let Ok(instance_id) = (unsafe { instance_id_pwstr.to_string() }) else {
                continue;
            };
            let Some(target) = sessions.iter().find(|s| s.instance_id == instance_id) else {
                continue;
            };
            let Ok(vol) = control.cast::<ISimpleAudioVolume>() else {
                continue;
            };
            if let Err(e) = unsafe { vol.SetMute(false, std::ptr::null()) } {
                tracing::debug!(?e, exe = %target.exe_basename, "SetMute(false) failed");
            } else {
                tracing::info!(exe = %target.exe_basename, "audio ducking: restored output session");
            }
        }
        Ok(())
    }

    unsafe fn session_manager() -> Result<IAudioSessionManager2> {
        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
        let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;
        let manager: IAudioSessionManager2 = device
            .Activate(CLSCTX_ALL, None)
            .map_err(|e| anyhow!("Activate(IAudioSessionManager2): {e}"))?;
        Ok(manager)
    }

    fn exe_basename_of(pid: u32) -> Option<String> {
        unsafe {
            let handle = OpenProcess(
                PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ,
                false,
                pid,
            )
            .ok()?;
            let mut buf = [0u16; 260];
            let len = GetModuleBaseNameW(handle, None, &mut buf);
            let _ = CloseHandle(handle);
            if len == 0 {
                return None;
            }
            Some(String::from_utf16_lossy(&buf[..len as usize]))
        }
    }
}

/// Default list of conferencing-app EXE names. Case-insensitive match on the basename
/// reported by Windows — we avoid the path so Store-installed vs classic-installed
/// Teams both resolve the same way. Users can override via the settings UI if a
/// rarer client comes along.
pub const DEFAULT_CONFERENCING_APPS: &[&str] = &[
    // Microsoft Teams — classic (Electron) + new WinUI client + Store variant.
    "Teams.exe",
    "ms-teams.exe",
    "msteams.exe",
    // Slack desktop.
    "slack.exe",
    // Discord stable + PTB + Canary.
    "Discord.exe",
    "DiscordPTB.exe",
    "DiscordCanary.exe",
    // Zoom.
    "Zoom.exe",
    "ZoomPhone.exe",
    // Google Meet (when installed as a PWA it runs under chrome.exe; we can't target
    // that without hitting every Chrome tab, so we intentionally leave it out).
    // Webex.
    "webex.exe",
    "WebexTeams.exe",
];

/// Return the default conferencing-app list as owned strings, for the
/// settings UI to display when the user has no override set.
/// Convenience wrapper so callers don't have to import the const +
/// re-string-ify it themselves.
pub fn default_conferencing_apps() -> Vec<String> {
    DEFAULT_CONFERENCING_APPS
        .iter()
        .map(|s| (*s).to_string())
        .collect()
}

// ── Validation for user-supplied EXE basenames ────────────────────────
//
// The conferencing-mute UI lets users add custom EXE basenames to the
// override list. The audio session matcher is already defensive
// (case-insensitive comparison against `GetModuleBaseNameW` output),
// but the UI-side input still wants validation:
//   1. To reject obvious mistakes (paths, wildcards, command lines)
//      before they hit settings.json.
//   2. To produce a consistent canonical form (`Teams.exe` not
//      `teams `, `Teams `, or `TEAMS.exe`) so duplicates collapse and
//      the displayed list reads cleanly.
//
// Validation is a pure function so it's directly testable. The Tauri
// command `commands::settings::validate_conferencing_app_name` calls
// it and returns the canonical form or a `ValidateAppNameError`.

/// Failure modes for `validate_conferencing_app_name`. Each variant
/// maps to a user-facing message in the settings UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidateAppNameError {
    /// Whitespace-only input.
    Empty,
    /// Contains a path separator (`/` or `\`), drive letter, quote,
    /// wildcard, or any character outside the conservative
    /// `[A-Za-z0-9._-]` set Windows EXE basenames can carry.
    InvalidCharacters,
    /// Length exceeds 64 chars after canonicalisation. Real EXE
    /// basenames are well under this; longer input is almost
    /// certainly a paste accident or a command line.
    TooLong,
    /// After canonicalisation, the name is already in the
    /// caller-supplied list (case-insensitive). Caller decides
    /// whether to surface this as a soft warning or a hard error.
    Duplicate,
}

/// Maximum accepted length for a canonicalised EXE basename. Real
/// Windows basenames almost never exceed this; we cap to keep
/// settings.json sane and to short-circuit clipboard-paste accidents
/// (a full command line, a URL, etc.).
const MAX_EXE_BASENAME_LEN: usize = 64;

/// Sanitise a user-supplied EXE basename and check it against an
/// existing list (case-insensitive de-duplication).
///
/// Behaviour:
///   * Trims leading/trailing whitespace.
///   * Rejects empty / whitespace-only input.
///   * Rejects path separators (`/` or `\`), drive letters, quotes,
///     wildcards (`*`, `?`), and any character outside `[A-Za-z0-9._-]`.
///   * Auto-appends `.exe` if the user typed e.g. `Teams` (most users
///     don't include the suffix). The audio session matcher only
///     accepts EXE basenames, not arbitrary process names.
///   * Caps the canonicalised length at 64 chars.
///   * If the canonical form already appears in `existing` (case-
///     insensitive), returns `ValidateAppNameError::Duplicate`.
///
/// Returns the canonical (post-sanitisation) form on success, ready to
/// `push` onto the override list.
pub fn validate_conferencing_app_name(
    input: &str,
    existing: &[String],
) -> Result<String, ValidateAppNameError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(ValidateAppNameError::Empty);
    }
    // Path separators, drive colons, quotes, wildcards, pipes,
    // angle brackets, semicolons — anything that's a sign of a
    // command line or a path. We allow only the conservative
    // basename character set.
    for ch in trimmed.chars() {
        let allowed = ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-');
        if !allowed {
            return Err(ValidateAppNameError::InvalidCharacters);
        }
    }
    // Auto-append `.exe` when missing. Case-insensitive check: a user
    // who types `Teams.EXE` shouldn't get a second `.exe` tacked on.
    let canonical = if trimmed.to_ascii_lowercase().ends_with(".exe") {
        trimmed.to_string()
    } else {
        format!("{trimmed}.exe")
    };
    if canonical.len() > MAX_EXE_BASENAME_LEN {
        return Err(ValidateAppNameError::TooLong);
    }
    let canonical_lower = canonical.to_ascii_lowercase();
    for existing_name in existing {
        if existing_name.to_ascii_lowercase() == canonical_lower {
            return Err(ValidateAppNameError::Duplicate);
        }
    }
    Ok(canonical)
}

#[cfg(test)]
mod validate_app_name_tests {
    //! Pure-function tests for the conferencing-mute UI's input
    //! validation. The audio session matcher is already case-
    //! insensitive and defensive against odd input, so these tests
    //! pin the UX-side rejection rules: what users see when they type
    //! a path, a wildcard, or a duplicate.

    use super::*;

    #[test]
    fn accepts_plain_exe_basename() {
        let r = validate_conferencing_app_name("Teams.exe", &[]).unwrap();
        assert_eq!(r, "Teams.exe");
    }

    #[test]
    fn auto_appends_exe_suffix() {
        let r = validate_conferencing_app_name("Teams", &[]).unwrap();
        assert_eq!(r, "Teams.exe");
    }

    #[test]
    fn does_not_double_append_exe() {
        // Even when the user types upper-case .EXE, we keep their
        // capitalisation but don't add a second suffix.
        let r = validate_conferencing_app_name("Teams.EXE", &[]).unwrap();
        assert_eq!(r, "Teams.EXE");
    }

    #[test]
    fn trims_surrounding_whitespace() {
        let r = validate_conferencing_app_name("  Teams.exe  ", &[]).unwrap();
        assert_eq!(r, "Teams.exe");
    }

    #[test]
    fn rejects_empty_input() {
        assert_eq!(
            validate_conferencing_app_name("", &[]),
            Err(ValidateAppNameError::Empty)
        );
        assert_eq!(
            validate_conferencing_app_name("   ", &[]),
            Err(ValidateAppNameError::Empty)
        );
    }

    #[test]
    fn rejects_paths_and_drive_letters() {
        for input in [
            r"C:\Program Files\Teams\Teams.exe",
            r"\\server\share\Teams.exe",
            "/usr/bin/teams",
            r"./Teams.exe",
            r"..\Teams.exe",
            "C:Teams.exe", // colon
        ] {
            assert_eq!(
                validate_conferencing_app_name(input, &[]),
                Err(ValidateAppNameError::InvalidCharacters),
                "expected InvalidCharacters for {input:?}"
            );
        }
    }

    #[test]
    fn rejects_wildcards() {
        for input in ["Teams*.exe", "Team?.exe", "*.exe", "Teams.*"] {
            assert_eq!(
                validate_conferencing_app_name(input, &[]),
                Err(ValidateAppNameError::InvalidCharacters),
                "expected InvalidCharacters for {input:?}"
            );
        }
    }

    #[test]
    fn rejects_command_line_and_quotes() {
        for input in [
            r#""Teams.exe""#,
            "Teams.exe --foo",
            "Teams.exe; rm",
            "Teams.exe|cat",
        ] {
            assert!(
                matches!(
                    validate_conferencing_app_name(input, &[]),
                    Err(ValidateAppNameError::InvalidCharacters)
                ),
                "expected InvalidCharacters for {input:?}"
            );
        }
    }

    #[test]
    fn rejects_oversized_input() {
        // 60 + 4 (".exe") = 64 (the cap, accepted).
        let cap_ok = "a".repeat(60);
        assert!(validate_conferencing_app_name(&cap_ok, &[]).is_ok());
        // 61 + 4 = 65 > 64 (rejected). Plain ASCII so no
        // multibyte-char surprises.
        let too_long = "a".repeat(61);
        assert_eq!(
            validate_conferencing_app_name(&too_long, &[]),
            Err(ValidateAppNameError::TooLong)
        );
    }

    #[test]
    fn deduplicates_case_insensitively() {
        let existing = vec!["Teams.exe".to_string()];
        // Typed without suffix, different case — still a duplicate
        // after canonicalisation.
        assert_eq!(
            validate_conferencing_app_name("teams", &existing),
            Err(ValidateAppNameError::Duplicate)
        );
        assert_eq!(
            validate_conferencing_app_name("TEAMS.EXE", &existing),
            Err(ValidateAppNameError::Duplicate)
        );
    }

    #[test]
    fn allows_dots_underscores_dashes() {
        // Real-world EXE basenames carry these.
        assert!(validate_conferencing_app_name("ms-teams.exe", &[]).is_ok());
        assert!(validate_conferencing_app_name("Some_App.exe", &[]).is_ok());
        assert!(validate_conferencing_app_name("App.123.exe", &[]).is_ok());
    }

    #[test]
    fn default_list_helper_returns_built_in_set() {
        let defaults = default_conferencing_apps();
        // Pin a few canonical entries so the UI always has these to
        // show. Don't pin the full count — adding a new vendor
        // shouldn't break this test.
        assert!(defaults.iter().any(|s| s == "Teams.exe"));
        assert!(defaults.iter().any(|s| s == "Discord.exe"));
        assert!(defaults.iter().any(|s| s == "Zoom.exe"));
        assert!(defaults.iter().any(|s| s == "slack.exe"));
        assert!(defaults.iter().any(|s| s == "webex.exe"));
        // Sanity: no duplicates in the static list.
        let mut sorted: Vec<String> =
            defaults.iter().map(|s| s.to_ascii_lowercase()).collect();
        sorted.sort();
        let count = sorted.len();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            count,
            "DEFAULT_CONFERENCING_APPS must not contain case-insensitive duplicates"
        );
    }

    #[test]
    fn no_duplicate_on_disjoint_existing() {
        let existing = vec!["Discord.exe".to_string(), "Zoom.exe".to_string()];
        let r = validate_conferencing_app_name("Teams.exe", &existing).unwrap();
        assert_eq!(r, "Teams.exe");
    }
}
