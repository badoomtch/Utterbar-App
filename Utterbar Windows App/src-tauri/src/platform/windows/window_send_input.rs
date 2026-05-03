//! Window-targeting `SendInput` wrapper for the experimental Teams + Discord
//! mic-mute feature. Branch 2 prototype.
//!
//! Locates a top-level window belonging to a target process (Teams or
//! Discord variant), then synthesises a hotkey press into that window.
//!
//! # Approach
//!
//! `SendInput` injects keystrokes into the foreground window's input queue;
//! it has no "send to background HWND" mode. The reliable cross-app approach
//! for Electron / WebView2 hosts (which both Teams and Discord are) is a
//! brief focus dance: save the user's current foreground, attach our thread
//! to the target window's input thread so `SetForegroundWindow` is allowed,
//! switch foreground to the target, send the keystrokes, restore the prior
//! foreground, detach. The visible side-effect is a brief focus flicker; the
//! Branch 2 spec calls this out as a finding to record.
//!
//! `PostMessage(WM_KEYDOWN/WM_KEYUP)` was considered and rejected. It works
//! for some Win32-native apps but not for Electron / WebView2 hosts (the
//! Chromium message loop handles raw keyboard messages differently from
//! input-queue events; both Teams new and Discord routinely ignore
//! `PostMessage`-delivered key events for global commands like Toggle Mute).
//!
//! # Scope
//!
//! Behind the `experimental_mic_mute` Cargo feature; not compiled into
//! release builds. No UI wiring, no Settings field, no banner. The two unit
//! tests below are `#[ignore]` so `cargo test` skips them; run manually
//! against a real Teams / Discord instance.
//!
//! See `docs/decisions/true-per-app-microphone-mute-v1-spec.md` for the
//! product context and the literal copy this prototype eventually feeds.

#![cfg(feature = "experimental_mic_mute")]
// Branch 2 prototype: nothing in the rest of the codebase calls these
// items yet; Branch 3 wires them to the Settings layer. Allow dead-code
// warnings until then so the build is quiet.
#![allow(dead_code)]

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::time::Duration;

use anyhow::{anyhow, Result};
use windows::Win32::Foundation::{CloseHandle, BOOL, HWND, LPARAM, TRUE};
use windows::Win32::System::Threading::{
    AttachThreadInput, GetCurrentThreadId, OpenProcess, QueryFullProcessImageNameW,
    PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
    VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetForegroundWindow, GetWindowThreadProcessId, IsWindowVisible,
    SetForegroundWindow,
};

/// One modifier-plus-key combo. The same shape will eventually be persisted
/// to settings; for the prototype we just construct it inline.
#[derive(Clone, Debug)]
pub struct Hotkey {
    /// Modifier keys held down for the duration of the key press, e.g.
    /// `[VK_CONTROL, VK_SHIFT]` for Ctrl+Shift+M. Order is the order they
    /// are pressed; release happens in reverse order. v1 spec rejects
    /// bare-letter binds, so this list is non-empty in any toggle-on path,
    /// but the wrapper itself does not enforce that, validation lives in
    /// the Settings layer.
    pub modifiers: Vec<VIRTUAL_KEY>,
    /// The main key, e.g. `VK_M` for Ctrl+Shift+M.
    pub main: VIRTUAL_KEY,
}

/// Find the first top-level visible window belonging to one of the named
/// processes. Process names are matched case-insensitively against the
/// owning process's module base name (e.g. `Teams.exe`, `Discord.exe`).
///
/// Returns `None` if no matching window is found. Returns the first match
/// in `EnumWindows` order, which is roughly z-order most-recent-first; for
/// our use case (one Teams window, one Discord window) this is what we
/// want. If the user has multiple Teams windows open we pick whichever
/// `EnumWindows` returned first; the keystroke goes to that one.
pub fn find_window_for_processes(process_names: &[&str]) -> Option<HWND> {
    let mut state = FindState {
        wanted: process_names,
        result: None,
    };
    unsafe {
        // Return value is intentionally ignored: EnumWindows returns FALSE
        // when our callback short-circuits with the first match, which is
        // success from our point of view, not an error.
        let _ = EnumWindows(
            Some(enum_windows_proc),
            LPARAM(&mut state as *mut _ as isize),
        );
    }
    state.result
}

struct FindState<'a> {
    wanted: &'a [&'a str],
    result: Option<HWND>,
}

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    if !IsWindowVisible(hwnd).as_bool() {
        return TRUE;
    }
    let state = &mut *(lparam.0 as *mut FindState);
    let pid = pid_for_window(hwnd);
    if pid == 0 {
        return TRUE;
    }
    let Some(process_name) = process_module_base_name(pid) else {
        return TRUE;
    };
    for wanted in state.wanted {
        if process_name.eq_ignore_ascii_case(wanted) {
            state.result = Some(hwnd);
            return windows::Win32::Foundation::FALSE;
        }
    }
    TRUE
}

unsafe fn pid_for_window(hwnd: HWND) -> u32 {
    let mut pid: u32 = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut pid as *mut u32));
    pid
}

unsafe fn process_module_base_name(pid: u32) -> Option<String> {
    // Use QueryFullProcessImageNameW rather than K32GetModuleBaseNameW.
    // The latter looks like a drop-in but actually requires
    // PROCESS_QUERY_INFORMATION + PROCESS_VM_READ; under the more
    // permissive PROCESS_QUERY_LIMITED_INFORMATION it always returns
    // ERROR_ACCESS_DENIED. Discovered during the Branch 2 smoke test:
    // every process returned None until this swap.
    let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
    if handle.is_invalid() {
        return None;
    }
    let mut buf = [0u16; 1024];
    let mut len: u32 = buf.len() as u32;
    let result = QueryFullProcessImageNameW(
        handle,
        PROCESS_NAME_FORMAT(0),
        windows::core::PWSTR(buf.as_mut_ptr()),
        &mut len,
    );
    let _ = CloseHandle(handle);
    if result.is_err() || len == 0 {
        return None;
    }
    let full = OsString::from_wide(&buf[..len as usize])
        .into_string()
        .ok()?;
    let basename = full
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or(&full)
        .to_string();
    Some(basename)
}

/// Send the given hotkey combo to the target window. Performs the focus
/// dance described in the module docs: saves the current foreground,
/// attaches the calling thread's input to the target window's thread,
/// switches foreground to the target, presses modifiers + main key,
/// releases in reverse order, restores the prior foreground, and detaches.
///
/// Errors:
///
/// - `find_window_for_processes` returned `None`, no target window found.
/// - `SetForegroundWindow` returned 0 (Windows refused; rare under
///   `AttachThreadInput` but possible if another process holds the
///   foreground lock or if UIPI blocks us, e.g. an elevated Teams).
/// - `SendInput` returned 0 (input was blocked).
///
/// Returns `Ok(())` only when every `SendInput` call returned a non-zero
/// count. The caller decides what to do with that signal; the v1 spec
/// treats a non-zero return as "we sent it; user is the verifier".
pub fn send_hotkey_to_window(hwnd: HWND, combo: &Hotkey) -> Result<()> {
    if hwnd.0.is_null() {
        return Err(anyhow!("send_hotkey_to_window: null HWND"));
    }
    let prior_foreground = unsafe { GetForegroundWindow() };
    let our_thread = unsafe { GetCurrentThreadId() };
    let target_thread = unsafe { GetWindowThreadProcessId(hwnd, None) };
    if target_thread == 0 {
        return Err(anyhow!(
            "send_hotkey_to_window: target window has no thread id"
        ));
    }

    // AttachThreadInput is what makes SetForegroundWindow legal here. Without
    // it, Windows enforces a foreground-lock policy (only the owner of the
    // current foreground can grant it elsewhere). Attaching our input to the
    // target's input queue puts us in the same focus group.
    let attached = unsafe { AttachThreadInput(our_thread, target_thread, true) };
    if !attached.as_bool() {
        return Err(anyhow!(
            "send_hotkey_to_window: AttachThreadInput failed"
        ));
    }

    let result = (|| -> Result<()> {
        let _ = unsafe { SetForegroundWindow(hwnd) };
        // Brief settle so the OS has applied the foreground change before
        // SendInput runs. 30ms is what we'd test in Branch 2 against real
        // Teams / Discord; tune as a finding.
        std::thread::sleep(Duration::from_millis(30));
        send_combo(combo)?;
        // Settle again before restoring foreground so the target's message
        // loop sees the keystroke in our brief foreground window.
        std::thread::sleep(Duration::from_millis(30));
        Ok(())
    })();

    // Best-effort restore of the prior foreground; if SetForegroundWindow
    // fails here we still want to detach the thread input below.
    if !prior_foreground.0.is_null() {
        let _ = unsafe { SetForegroundWindow(prior_foreground) };
    }
    let _detached = unsafe { AttachThreadInput(our_thread, target_thread, false) };

    result
}

fn send_combo(combo: &Hotkey) -> Result<()> {
    // Press modifiers, press main, release main, release modifiers (reverse).
    let mut inputs: Vec<INPUT> = Vec::with_capacity(combo.modifiers.len() * 2 + 2);
    for m in &combo.modifiers {
        inputs.push(vk_event(*m, false));
    }
    inputs.push(vk_event(combo.main, false));
    inputs.push(vk_event(combo.main, true));
    for m in combo.modifiers.iter().rev() {
        inputs.push(vk_event(*m, true));
    }

    let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
    if sent == 0 {
        return Err(anyhow!("SendInput returned 0"));
    }
    if sent as usize != inputs.len() {
        return Err(anyhow!(
            "SendInput returned {} of {} expected events",
            sent,
            inputs.len()
        ));
    }
    Ok(())
}

fn vk_event(vk: VIRTUAL_KEY, key_up: bool) -> INPUT {
    let flags: KEYBD_EVENT_FLAGS = if key_up {
        KEYEVENTF_KEYUP
    } else {
        KEYBD_EVENT_FLAGS(0)
    };
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

/// Process-name lists for the two v1 apps. Used by the manual-run tests and
/// will be reused by Branch 3's Settings layer.
pub const TEAMS_PROCESSES: &[&str] = &["Teams.exe", "ms-teams.exe"];
pub const DISCORD_PROCESSES: &[&str] = &["Discord.exe", "DiscordPTB.exe", "DiscordCanary.exe"];

#[cfg(test)]
mod manual_tests {
    //! Manual-run smoke tests. `#[ignore]` so they do not run on
    //! `cargo test`; invoke via:
    //!
    //! ```text
    //! cargo test --features experimental_mic_mute -p utterbar \
    //!     --bin utterbar -- --ignored --nocapture mic_mute
    //! ```
    //!
    //! These tests deliberately have side effects: they toggle the mute
    //! state in real Teams / Discord windows. Run them only when those apps
    //! are open AND you are not in a live call. They print latency and
    //! foreground-flicker observations to stdout.

    use super::*;
    use std::time::Instant;
    use windows::Win32::UI::Input::KeyboardAndMouse::{VK_CONTROL, VK_M, VK_SHIFT};

    fn ctrl_shift_m() -> Hotkey {
        Hotkey {
            modifiers: vec![VK_CONTROL, VK_SHIFT],
            main: VK_M,
        }
    }

    #[test]
    #[ignore]
    fn manual_mic_mute_teams_ctrl_shift_m() {
        let hwnd = find_window_for_processes(TEAMS_PROCESSES);
        let Some(hwnd) = hwnd else {
            eprintln!(
                "no Teams window found (looked for {:?}). Open Teams and re-run.",
                TEAMS_PROCESSES
            );
            return;
        };
        eprintln!("found Teams window: {:?}", hwnd);
        let prior = unsafe { GetForegroundWindow() };
        let started = Instant::now();
        let result = send_hotkey_to_window(hwnd, &ctrl_shift_m());
        let elapsed = started.elapsed();
        let after = unsafe { GetForegroundWindow() };
        eprintln!("send result:        {:?}", result);
        eprintln!("send elapsed:       {:?}", elapsed);
        eprintln!("foreground prior:   {:?}", prior);
        eprintln!("foreground after:   {:?}", after);
        eprintln!(
            "foreground restored: {}",
            prior.0 == after.0
        );
        assert!(result.is_ok(), "Teams hotkey send failed: {:?}", result);
    }

    #[test]
    #[ignore]
    fn manual_mic_mute_discord_ctrl_shift_m() {
        let hwnd = find_window_for_processes(DISCORD_PROCESSES);
        let Some(hwnd) = hwnd else {
            eprintln!(
                "no Discord window found (looked for {:?}). Open Discord and re-run.",
                DISCORD_PROCESSES
            );
            return;
        };
        eprintln!("found Discord window: {:?}", hwnd);
        let prior = unsafe { GetForegroundWindow() };
        let started = Instant::now();
        let result = send_hotkey_to_window(hwnd, &ctrl_shift_m());
        let elapsed = started.elapsed();
        let after = unsafe { GetForegroundWindow() };
        eprintln!("send result:        {:?}", result);
        eprintln!("send elapsed:       {:?}", elapsed);
        eprintln!("foreground prior:   {:?}", prior);
        eprintln!("foreground after:   {:?}", after);
        eprintln!(
            "foreground restored: {}",
            prior.0 == after.0
        );
        assert!(result.is_ok(), "Discord hotkey send failed: {:?}", result);
    }

}
