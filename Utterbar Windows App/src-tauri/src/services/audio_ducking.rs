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

    /// Small RAII guard so failures mid-function still balance CoUninitialize.
    struct ComGuard;
    impl ComGuard {
        fn new() -> Self {
            // Safe even if COM is already initialised on this thread — returns an HRESULT
            // we can ignore. Matches the pattern used in services/audio/loopback.rs.
            unsafe {
                let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            }
            ComGuard
        }
    }
    impl Drop for ComGuard {
        fn drop(&mut self) {
            unsafe { CoUninitialize() };
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
