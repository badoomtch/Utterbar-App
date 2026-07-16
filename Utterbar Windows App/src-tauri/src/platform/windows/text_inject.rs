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
    CloseClipboard, EmptyClipboard, GetClipboardData, IsClipboardFormatAvailable, OpenClipboard,
    SetClipboardData,
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
        } => match paste_via_clipboard(text, restore_clipboard, fallback_to_keystrokes) {
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
    if sent == 0 {
        return Err(anyhow!("SendInput(Enter) returned 0"));
    }
    Ok(())
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

fn paste_via_clipboard(text: &str, restore: bool, _detect_failure: bool) -> Result<()> {
    let prior = if restore {
        read_clipboard_text().ok()
    } else {
        None
    };

    set_clipboard_text(text)?;
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
        // Restore off-thread, but only after we have evidence the host has
        // consumed the paste. The previous fixed-500ms timer caused the
        // "pastes the thing I already had copied" bug when the host's
        // WM_PASTE handler ran later than 500ms (Office cold start, slow
        // Electron apps, modal pump): we'd write `prev` back to the
        // clipboard before the host read it, and the host then pasted the
        // old value.
        //
        // The honest signal we have is the clipboard itself. After Ctrl+V
        // the host either (a) reads the clipboard and moves on, leaving
        // our transcript in place, or (b) replaces it (some apps do
        // copy-on-paste tricks), or (c) the user explicitly copies
        // something else. Cases (b) and (c) mean we MUST NOT restore —
        // the clipboard no longer contains our transcript, so writing
        // `prev` would clobber whatever's there now. Case (a) is the only
        // one where restoring is safe.
        //
        // We poll every 50ms up to 900ms. As soon as the clipboard
        // contents differ from our transcript, the host has consumed the
        // paste (or the user moved on) — stop polling and bail without
        // restoring. If we hit the ceiling with our transcript still on
        // the clipboard, the host most likely already pasted but didn't
        // disturb the clipboard (the common case for Office, most
        // browsers, terminals); it's then safe to write `prev` back.
        //
        // The ceiling used to be 2000ms — picked when "Office cold start"
        // was the worst observed paste-handler latency at ~500ms. Users
        // reported the original clipboard staying gone for far too long
        // after every dictation; 2000 → 900 → 600ms in stages. 600ms
        // gives ~100ms margin over the observed ~500ms cold-start window
        // while returning the prior clipboard sooner on the typical-host
        // path (a plain host that doesn't touch the clipboard on paste
        // holds our transcript for this whole window before we restore).
        // POLL_MS stays at 50 so an overwrite during the wait is caught
        // fast. This is the single dial: raise it if a slow host starts
        // pasting the OLD clipboard; lower it to return the prior
        // clipboard even faster at the cost of that margin.
        let our_text = text.to_string();
        std::thread::spawn(move || {
            const POLL_MS: u64 = 50;
            const MAX_MS: u64 = 600;
            let mut elapsed = 0u64;
            // Initial settle — the host's paste handler almost always
            // fires inside this window. Skipping this and polling
            // immediately can race against fast hosts that consume the
            // paste in <10ms.
            sleep(Duration::from_millis(POLL_MS));
            elapsed += POLL_MS;
            while elapsed < MAX_MS {
                match read_clipboard_text() {
                    Ok(current) if current == our_text => {
                        // Host hasn't touched the clipboard yet. Wait longer.
                        sleep(Duration::from_millis(POLL_MS));
                        elapsed += POLL_MS;
                    }
                    Ok(_) => {
                        // Clipboard changed — host (or user) overwrote our
                        // transcript. Restoring now would clobber that. Bail.
                        tracing::info!(
                            "clipboard restore: skipped, clipboard no longer contains our transcript"
                        );
                        return;
                    }
                    Err(_) => {
                        // Couldn't read (another app holding the clipboard,
                        // transient OpenClipboard failure). Retry next tick.
                        sleep(Duration::from_millis(POLL_MS));
                        elapsed += POLL_MS;
                    }
                }
            }
            // Hit the ceiling with our transcript still on the clipboard.
            // Host probably pasted without disturbing it. Safe to restore.
            if let Err(e) = set_clipboard_text(&prev) {
                tracing::warn!(?e, "failed to restore clipboard");
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
        if sent == 0 {
            return Err(anyhow!("SendInput returned 0"));
        }
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
    if sent == 0 {
        return Err(anyhow!("SendInput(Ctrl+V) returned 0"));
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
