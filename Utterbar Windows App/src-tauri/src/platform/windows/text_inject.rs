//! Windows text delivery.
//!
//!   1. **Paste** — put text on the clipboard (CF_UNICODETEXT) then synthesize Ctrl+V via
//!      `SendInput`. Optionally snapshot-and-restore the previous clipboard payload.
//!   2. **Keystrokes** — synthesize Unicode key events, one per UTF-16 code unit.

use std::thread::sleep;
use std::time::Duration;

use anyhow::{anyhow, Result};
use windows::Win32::Foundation::{HANDLE, HGLOBAL, HWND};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, GetClipboardSequenceNumber,
    IsClipboardFormatAvailable, OpenClipboard, SetClipboardData,
};
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
use windows::Win32::System::Threading::GetCurrentProcessId;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
    KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, VIRTUAL_KEY, VK_CONTROL, VK_RETURN, VK_SHIFT, VK_V,
};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

use crate::services::text_inject::Strategy;

// CF_UNICODETEXT from winuser.h — reproduced here so we don't need the Ole feature flag.
const CF_UNICODETEXT: u32 = 13;

pub fn deliver(text: &str, strategy: Strategy) -> Result<()> {
    if text.is_empty() {
        return Ok(());
    }
    // Never paste into our own windows. Without this, any Utterbar-hosted surface
    // that already inserts the transcript in JS (onboarding's Ready step, the
    // history reprocess panel, settings test fields) would receive the text twice:
    // once from the Ctrl+V synthesis below, once from the DICTATION_RESULT event
    // listener. Also covers the general case of "stop recording whilst focused in
    // Utterbar" — the user explicitly switched away if they wanted paste.
    if foreground_is_self() {
        tracing::info!("text_inject: skipping delivery — Utterbar itself is foreground");
        return Ok(());
    }
    match strategy {
        Strategy::Paste {
            restore_clipboard,
            fallback_to_keystrokes,
        } => match paste_via_clipboard(text, restore_clipboard) {
            Ok(()) => Ok(()),
            Err(e) if fallback_to_keystrokes => {
                tracing::warn!(?e, "paste failed, falling back to keystrokes");
                send_keystrokes(text)
            }
            Err(e) => Err(e),
        },
        Strategy::Keystrokes => send_keystrokes(text),
    }
}

/// Non-blocking check for the Shift key. `GetAsyncKeyState` returns a SHORT where the
/// top bit is set if the key is currently down. We don't use the low bit (which means
/// "pressed since last call") because it clears on read and would race with the tauri
/// event loop's own shift polling.
pub fn shift_is_held() -> bool {
    unsafe { (GetAsyncKeyState(VK_SHIFT.0 as i32) as u16) & 0x8000 != 0 }
}

/// Press + release Return. Called from the auto-send flow after a successful paste, so
/// apps like Teams / Slack / Discord submit the line without the user hitting Enter.
pub fn send_enter() -> Result<()> {
    let inputs = [vk_event(VK_RETURN, false), vk_event(VK_RETURN, true)];
    let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
    ensure_all_inputs_sent(sent, inputs.len(), "Enter")
}

/// Returns `true` when the OS-level foreground window belongs to our own process.
/// Used to avoid pasting a transcript into an Utterbar-hosted textarea that is
/// already receiving the result via a JS event listener.
fn foreground_is_self() -> bool {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return false;
        }
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid as *mut u32));
        pid != 0 && pid == GetCurrentProcessId()
    }
}

fn paste_via_clipboard(text: &str, restore: bool) -> Result<()> {
    let prior = if restore {
        read_clipboard_text_with_retry().ok()
    } else {
        None
    };

    set_clipboard_text(text)?;
    let transcript_sequence = unsafe { GetClipboardSequenceNumber() };
    // Give the OS a beat to let SetClipboardData propagate to listeners before the
    // paste event reads it. 40ms is safe across every app we've tested; 15ms wasn't
    // enough for heavier Electron clients (Discord, VS Code, Slack) on a slow tick,
    // which occasionally read the *old* clipboard during Ctrl+V.
    sleep(Duration::from_millis(40));
    // send_ctrl_v returns Err when SendInput returns 0 — which is how Windows signals
    // that the input was blocked (elevated target window, UIPI, etc.). That's the one
    // honest signal we have for "paste didn't land". The caller's keystroke fallback
    // then retries via per-character Unicode SendInput, which goes through a different
    // injection path and sometimes succeeds where Ctrl+V did not. We deliberately do NOT
    // try to detect silent failure (clipboard still intact) — most apps leave the
    // clipboard alone on paste, so that check false-positives everywhere.
    send_ctrl_v()?;

    if let Some(prev) = prior {
        // A paste does not normally mutate the clipboard, so comparing its
        // text cannot tell us whether the target consumed Ctrl+V. Instead,
        // wait long enough for slow hosts, then restore only when the
        // clipboard sequence number still matches the value created for this
        // transcript. Any new user or application copy, including a non-text
        // clipboard payload, changes that number and is left alone.
        std::thread::spawn(move || {
            const RESTORE_DELAY_MS: u64 = 1_500;
            sleep(Duration::from_millis(RESTORE_DELAY_MS));
            let current_sequence = unsafe { GetClipboardSequenceNumber() };
            if should_restore_clipboard(transcript_sequence, current_sequence) {
                if let Err(e) = set_clipboard_text(&prev) {
                    tracing::warn!(?e, "failed to restore clipboard");
                }
            } else {
                tracing::info!(
                    transcript_sequence,
                    current_sequence,
                    "clipboard restore skipped because clipboard changed"
                );
            }
        });
    }
    Ok(())
}

fn send_keystrokes(text: &str) -> Result<()> {
    let utf16: Vec<u16> = text.encode_utf16().collect();
    let mut inputs: Vec<INPUT> = Vec::with_capacity(utf16.len() * 2);
    for code in utf16 {
        inputs.push(unicode_key(code, false));
        inputs.push(unicode_key(code, true));
    }

    for chunk in inputs.chunks(64) {
        let sent = unsafe { SendInput(chunk, std::mem::size_of::<INPUT>() as i32) };
        ensure_all_inputs_sent(sent, chunk.len(), "Unicode text")?;
        sleep(Duration::from_millis(1));
    }
    Ok(())
}

fn unicode_key(code: u16, key_up: bool) -> INPUT {
    let mut flags = KEYEVENTF_UNICODE;
    if key_up {
        flags |= KEYEVENTF_KEYUP;
    }
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(0),
                wScan: code,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn send_ctrl_v() -> Result<()> {
    let inputs = [
        vk_event(VK_CONTROL, false),
        vk_event(VK_V, false),
        vk_event(VK_V, true),
        vk_event(VK_CONTROL, true),
    ];
    let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
    ensure_all_inputs_sent(sent, inputs.len(), "Ctrl+V")
}

fn ensure_all_inputs_sent(sent: u32, expected: usize, operation: &str) -> Result<()> {
    if sent as usize == expected {
        Ok(())
    } else {
        Err(anyhow!(
            "SendInput({operation}) sent {sent} of {expected} expected events"
        ))
    }
}

fn should_restore_clipboard(transcript_sequence: u32, current_sequence: u32) -> bool {
    transcript_sequence != 0 && transcript_sequence == current_sequence
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

// ---------- clipboard helpers (text-only) ----------

struct ClipGuard;
impl ClipGuard {
    fn open() -> Result<Self> {
        unsafe { OpenClipboard(HWND(std::ptr::null_mut())) }
            .map_err(|e| anyhow!("OpenClipboard: {e}"))?;
        Ok(Self)
    }
}
impl Drop for ClipGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseClipboard();
        }
    }
}

pub fn read_clipboard_text() -> Result<String> {
    unsafe {
        // Newer windows-rs exposes this as Result<(), E> on success path; map Err to the
        // "no text available" case so callers get a uniform error.
        IsClipboardFormatAvailable(CF_UNICODETEXT)
            .map_err(|_| anyhow!("no unicode text on clipboard"))?;
        let _guard = ClipGuard::open()?;

        let handle: HANDLE =
            GetClipboardData(CF_UNICODETEXT).map_err(|e| anyhow!("GetClipboardData: {e}"))?;
        let h = HGLOBAL(handle.0);
        let ptr = GlobalLock(h) as *const u16;
        if ptr.is_null() {
            return Err(anyhow!("GlobalLock returned null"));
        }

        let mut len = 0usize;
        while *ptr.add(len) != 0 {
            len += 1;
            if len > 2_000_000 {
                break;
            }
        }
        let slice = std::slice::from_raw_parts(ptr, len);
        let s = String::from_utf16_lossy(slice);
        let _ = GlobalUnlock(h);
        Ok(s)
    }
}

fn read_clipboard_text_with_retry() -> Result<String> {
    const ATTEMPTS: usize = 6;
    const RETRY_DELAY_MS: u64 = 25;

    let mut last_error = None;
    for attempt in 0..ATTEMPTS {
        match read_clipboard_text() {
            Ok(text) => return Ok(text),
            Err(error) => {
                last_error = Some(error);
                if attempt + 1 < ATTEMPTS {
                    sleep(Duration::from_millis(RETRY_DELAY_MS));
                }
            }
        }
    }
    Err(last_error.expect("clipboard retry has at least one attempt"))
}

pub fn set_clipboard_text(text: &str) -> Result<()> {
    let mut utf16: Vec<u16> = text.encode_utf16().collect();
    utf16.push(0);
    let size_bytes = utf16.len() * std::mem::size_of::<u16>();

    unsafe {
        let _guard = ClipGuard::open()?;
        EmptyClipboard().map_err(|e| anyhow!("EmptyClipboard: {e}"))?;

        let h_mem: HGLOBAL =
            GlobalAlloc(GMEM_MOVEABLE, size_bytes).map_err(|e| anyhow!("GlobalAlloc: {e}"))?;
        let dst = GlobalLock(h_mem) as *mut u16;
        if dst.is_null() {
            return Err(anyhow!("GlobalLock returned null"));
        }
        std::ptr::copy_nonoverlapping(utf16.as_ptr(), dst, utf16.len());
        let _ = GlobalUnlock(h_mem);

        // Clipboard takes ownership of h_mem on success; don't free it.
        SetClipboardData(CF_UNICODETEXT, HANDLE(h_mem.0))
            .map_err(|e| anyhow!("SetClipboardData: {e}"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn send_input_requires_every_requested_event() {
        assert!(ensure_all_inputs_sent(4, 4, "test").is_ok());
        assert!(ensure_all_inputs_sent(0, 4, "test").is_err());
        assert!(ensure_all_inputs_sent(3, 4, "test").is_err());
    }

    #[test]
    fn clipboard_restore_only_runs_when_sequence_is_unchanged() {
        assert!(should_restore_clipboard(42, 42));
        assert!(!should_restore_clipboard(42, 43));
        assert!(!should_restore_clipboard(0, 0));
    }
}
