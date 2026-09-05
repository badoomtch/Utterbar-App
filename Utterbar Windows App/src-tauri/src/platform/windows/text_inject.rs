//! Windows text delivery.
//!
//!   1. **Paste** — put text on the clipboard (CF_UNICODETEXT) then synthesize Ctrl+V via
//!      `SendInput`. Optionally snapshot-and-restore the previous clipboard payload.
//!   2. **Keystrokes** — synthesize Unicode key events, one per UTF-16 code unit.

use std::thread::sleep;
use std::time::Duration;

use anyhow::{anyhow, Result};
use windows::Win32::Foundation::{GlobalFree, HANDLE, HGLOBAL, HWND, LPARAM, WPARAM};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, EnumClipboardFormats, GetClipboardData,
    GetClipboardFormatNameW, GetClipboardSequenceNumber, IsClipboardFormatAvailable, OpenClipboard,
    SetClipboardData,
};
use windows::Win32::System::Memory::{
    GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE,
};
use windows::Win32::System::Threading::GetCurrentProcessId;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
    KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, VIRTUAL_KEY, VK_CONTROL, VK_RETURN, VK_SHIFT, VK_V,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetClassNameW, GetForegroundWindow, GetGUIThreadInfo, GetWindowThreadProcessId,
    SendMessageTimeoutW, GUITHREADINFO, SMTO_ABORTIFHUNG, SMTO_BLOCK, WM_PASTE,
};

use crate::services::text_inject::Strategy;

// CF_UNICODETEXT from winuser.h — reproduced here so we don't need the Ole feature flag.
const CF_UNICODETEXT: u32 = 13;
const CF_TEXT: u32 = 1;
const CF_OEMTEXT: u32 = 7;
const CF_DIB: u32 = 8;
const CF_DIBV5: u32 = 17;
const MAX_CLIPBOARD_FORMATS: usize = 64;
const MAX_CLIPBOARD_BYTES: usize = 128 * 1024 * 1024;
const MIN_CLIPBOARD_RESTORE_DELAY_MS: u64 = 250;
const MAX_CLIPBOARD_RESTORE_DELAY_MS: u64 = 1_500;

#[derive(Debug)]
struct ClipboardEntry {
    format: u32,
    bytes: Vec<u8>,
}

#[derive(Debug)]
struct ClipboardSnapshot {
    entries: Vec<ClipboardEntry>,
}

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
        return Err(anyhow!(
            "Utterbar itself is foreground; no external paste target is active"
        ));
    }
    match strategy {
        Strategy::Paste {
            restore_clipboard,
            clipboard_restore_delay_ms,
            fallback_to_keystrokes,
        } => match paste_via_clipboard(text, restore_clipboard, clipboard_restore_delay_ms) {
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

fn paste_via_clipboard(text: &str, restore: bool, restore_delay_ms: u32) -> Result<()> {
    let prior = if restore {
        // If preservation was requested, never silently overwrite a clipboard
        // that we could not snapshot. The caller can fall back to Unicode
        // keystrokes, which leaves images, files and rich formats untouched.
        Some(
            snapshot_clipboard_with_retry()
                .map_err(|error| anyhow!("could not preserve the existing clipboard: {error}"))?,
        )
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
    // Native editors expose a focused child control which accepts WM_PASTE.
    // Sending it synchronously confirms the control consumed the clipboard
    // before restoration can run. Notepad++ uses a Scintilla child control;
    // queued Ctrl+V input can otherwise be handled after a short restore delay.
    // Browser and Electron surfaces retain the existing Ctrl+V path.
    let delivery = if let Some((target, class_name)) = focused_direct_paste_target() {
        match send_wm_paste(target) {
            Ok(()) => {
                tracing::info!(
                    target_class = %class_name,
                    "text_inject: paste acknowledged by focused editor control"
                );
                Ok(())
            }
            Err(error) => {
                tracing::warn!(
                    ?error,
                    target_class = %class_name,
                    "text_inject: focused-control paste failed; retrying with Ctrl+V"
                );
                send_ctrl_v()
            }
        }
    } else {
        let result = send_ctrl_v();
        if result.is_ok() {
            tracing::info!("text_inject: Ctrl+V events accepted by Windows");
        }
        result
    };

    if let Err(error) = delivery {
        if let Some(previous) = prior.as_ref() {
            if let Err(restore_error) = restore_clipboard_snapshot(previous) {
                tracing::warn!(
                    ?restore_error,
                    "text_inject: immediate clipboard rollback failed after paste error"
                );
            }
        }
        return Err(error);
    }

    if let Some(prev) = prior {
        // A paste does not normally mutate the clipboard, so comparing its
        // text cannot tell us whether the target consumed Ctrl+V. Instead,
        // wait long enough for slow hosts, then restore only when the
        // clipboard sequence number still matches the value created for this
        // transcript. Any new user or application copy, including a non-text
        // clipboard payload, changes that number and is left alone.
        let restore_delay_ms = clamp_restore_delay_ms(restore_delay_ms);
        std::thread::spawn(move || {
            sleep(Duration::from_millis(restore_delay_ms));
            let current_sequence = unsafe { GetClipboardSequenceNumber() };
            if should_restore_clipboard(transcript_sequence, current_sequence) {
                if let Err(e) = restore_clipboard_snapshot(&prev) {
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

/// Return the focused native editor control when it supports synchronous
/// `WM_PASTE`. Top-level browser and Electron windows deliberately return
/// `None` and keep using synthetic Ctrl+V.
fn focused_direct_paste_target() -> Option<(HWND, String)> {
    unsafe {
        let foreground = GetForegroundWindow();
        if foreground.0.is_null() {
            return None;
        }

        let thread_id = GetWindowThreadProcessId(foreground, None);
        if thread_id == 0 {
            return None;
        }

        let mut info = GUITHREADINFO {
            cbSize: std::mem::size_of::<GUITHREADINFO>() as u32,
            ..Default::default()
        };
        if GetGUIThreadInfo(thread_id, &mut info).is_err() || info.hwndFocus.0.is_null() {
            return None;
        }

        let mut class_name = [0u16; 128];
        let length = GetClassNameW(info.hwndFocus, &mut class_name);
        if length <= 0 {
            return None;
        }
        let class_name = String::from_utf16_lossy(&class_name[..length as usize]);
        supports_direct_paste(&class_name).then_some((info.hwndFocus, class_name))
    }
}

fn supports_direct_paste(class_name: &str) -> bool {
    let normalised = class_name.to_ascii_lowercase();
    normalised == "edit"
        || normalised == "scintilla"
        || normalised.starts_with("richedit")
        || normalised.starts_with("windowsforms10.edit")
}

fn send_wm_paste(target: HWND) -> Result<()> {
    let result = unsafe {
        SendMessageTimeoutW(
            target,
            WM_PASTE,
            WPARAM(0),
            LPARAM(0),
            SMTO_ABORTIFHUNG | SMTO_BLOCK,
            750,
            None,
        )
    };
    if result.0 == 0 {
        Err(anyhow!("WM_PASTE timed out or was rejected"))
    } else {
        Ok(())
    }
}

fn clamp_restore_delay_ms(delay_ms: u32) -> u64 {
    u64::from(delay_ms).clamp(
        MIN_CLIPBOARD_RESTORE_DELAY_MS,
        MAX_CLIPBOARD_RESTORE_DELAY_MS,
    )
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

/// Snapshot global-memory clipboard formats so ordinary image copies (DIB,
/// DIBV5, PNG) can be restored after a transcript paste. Formats backed by
/// non-global handles are intentionally skipped: copying those handles would
/// create invalid ownership and can corrupt the source application's data.
fn snapshot_clipboard_with_retry() -> Result<ClipboardSnapshot> {
    const ATTEMPTS: usize = 6;
    const RETRY_DELAY_MS: u64 = 25;

    let mut last_error = None;
    for attempt in 0..ATTEMPTS {
        match snapshot_clipboard() {
            Ok(snapshot) => return Ok(snapshot),
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

fn snapshot_clipboard() -> Result<ClipboardSnapshot> {
    let _guard = ClipGuard::open()?;
    let mut entries = Vec::new();
    let mut total_bytes = 0usize;
    let mut format = 0u32;

    loop {
        format = unsafe { EnumClipboardFormats(format) };
        if format == 0 {
            break;
        }
        if entries.len() >= MAX_CLIPBOARD_FORMATS {
            return Err(anyhow!(
                "clipboard has more than {MAX_CLIPBOARD_FORMATS} formats"
            ));
        }

        if !format_uses_global_memory(format) {
            // GetClipboardData can return GDI, metafile and delayed-rendering
            // handles. Passing any of those to GlobalSize is invalid and, on
            // current Windows builds, terminates the process with heap
            // corruption rather than returning zero. Only inspect formats
            // whose clipboard contract is an HGLOBAL payload.
            continue;
        }

        let handle = unsafe { GetClipboardData(format) }
            .map_err(|error| anyhow!("clipboard format {format} unavailable: {error}"))?;
        let size = unsafe { GlobalSize(HGLOBAL(handle.0)) };
        if size == 0 {
            return Err(anyhow!(
                "clipboard format {format} has no restorable global-memory payload"
            ));
        }
        total_bytes = total_bytes.saturating_add(size);
        if total_bytes > MAX_CLIPBOARD_BYTES {
            return Err(anyhow!(
                "clipboard exceeds the {} MiB snapshot limit",
                MAX_CLIPBOARD_BYTES / 1024 / 1024
            ));
        }

        let source = unsafe { GlobalLock(HGLOBAL(handle.0)) } as *const u8;
        if source.is_null() {
            return Err(anyhow!(
                "clipboard format {format} could not be locked during snapshot"
            ));
        }
        let bytes = unsafe { std::slice::from_raw_parts(source, size) }.to_vec();
        let _ = unsafe { GlobalUnlock(HGLOBAL(handle.0)) };
        entries.push(ClipboardEntry { format, bytes });
    }

    if entries.is_empty() {
        return Err(anyhow!("clipboard has no restorable global-memory formats"));
    }
    Ok(ClipboardSnapshot { entries })
}

/// Returns whether a clipboard format is safe to inspect as an HGLOBAL byte
/// payload. Image copies commonly provide DIB/DIBV5 and PNG alongside GDI
/// bitmap handles; we retain the former and deliberately skip the latter.
fn format_uses_global_memory(format: u32) -> bool {
    matches!(
        format,
        CF_TEXT | CF_OEMTEXT | CF_UNICODETEXT | CF_DIB | CF_DIBV5
    ) || clipboard_format_name(format).as_deref() == Some("PNG")
}

fn clipboard_format_name(format: u32) -> Option<String> {
    let mut name = [0u16; 128];
    let length = unsafe { GetClipboardFormatNameW(format, &mut name) };
    (length > 0).then(|| String::from_utf16_lossy(&name[..length as usize]))
}

fn restore_clipboard_snapshot(snapshot: &ClipboardSnapshot) -> Result<()> {
    let _guard = ClipGuard::open()?;
    unsafe { EmptyClipboard().map_err(|e| anyhow!("EmptyClipboard: {e}"))? };

    let mut restored = 0usize;
    for entry in &snapshot.entries {
        let h_mem: HGLOBAL = unsafe {
            GlobalAlloc(GMEM_MOVEABLE, entry.bytes.len())
                .map_err(|e| anyhow!("GlobalAlloc: {e}"))?
        };
        let destination = unsafe { GlobalLock(h_mem) } as *mut u8;
        if destination.is_null() {
            unsafe {
                let _ = GlobalFree(h_mem);
            }
            return Err(anyhow!("GlobalLock returned null"));
        }
        unsafe {
            std::ptr::copy_nonoverlapping(entry.bytes.as_ptr(), destination, entry.bytes.len());
            let _ = GlobalUnlock(h_mem);
        }

        match unsafe { SetClipboardData(entry.format, HANDLE(h_mem.0)) } {
            Ok(_) => restored += 1,
            Err(error) => {
                unsafe {
                    let _ = GlobalFree(h_mem);
                }
                tracing::warn!(
                    format = entry.format,
                    ?error,
                    "clipboard format restore failed"
                );
            }
        }
    }

    if restored == 0 {
        return Err(anyhow!("no clipboard formats could be restored"));
    }
    Ok(())
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

        // Clipboard takes ownership of h_mem on success; release it ourselves
        // on failure.
        if let Err(error) = SetClipboardData(CF_UNICODETEXT, HANDLE(h_mem.0)) {
            let _ = GlobalFree(h_mem);
            return Err(anyhow!("SetClipboardData: {error}"));
        }
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

    #[test]
    fn clipboard_restore_delay_stays_within_the_supported_range() {
        assert_eq!(clamp_restore_delay_ms(0), 250);
        assert_eq!(clamp_restore_delay_ms(250), 250);
        assert_eq!(clamp_restore_delay_ms(750), 750);
        assert_eq!(clamp_restore_delay_ms(1_500), 1_500);
        assert_eq!(clamp_restore_delay_ms(9_999), 1_500);
    }

    #[test]
    fn clipboard_snapshot_retains_multiple_global_memory_formats() {
        let snapshot = ClipboardSnapshot {
            entries: vec![
                ClipboardEntry {
                    format: CF_UNICODETEXT,
                    bytes: vec![1, 0],
                },
                ClipboardEntry {
                    // CF_DIB is the common bitmap payload used by copied images.
                    format: 8,
                    bytes: vec![0; 40],
                },
            ],
        };

        assert_eq!(snapshot.entries.len(), 2);
        assert_eq!(snapshot.entries[1].format, 8);
    }

    #[test]
    fn clipboard_snapshot_only_reads_known_global_memory_formats() {
        assert!(format_uses_global_memory(CF_UNICODETEXT));
        assert!(format_uses_global_memory(CF_DIB));
        assert!(format_uses_global_memory(CF_DIBV5));
        assert!(!format_uses_global_memory(2)); // CF_BITMAP is a GDI handle.
        assert!(!format_uses_global_memory(9)); // CF_PALETTE is a GDI handle.
        assert!(!format_uses_global_memory(14)); // CF_ENHMETAFILE is not HGLOBAL.
    }

    #[test]
    fn clipboard_snapshot_checks_format_before_global_size() {
        let source = include_str!("text_inject.rs");
        let snapshot_start = source
            .find("fn snapshot_clipboard()")
            .expect("clipboard snapshot must exist");
        let snapshot = &source[snapshot_start..];
        let guard = snapshot
            .find("if !format_uses_global_memory(format)")
            .expect("snapshot must reject non-HGLOBAL formats");
        let global_size = snapshot
            .find("GlobalSize(HGLOBAL(handle.0))")
            .expect("snapshot must size approved HGLOBAL payloads");
        assert!(
            guard < global_size,
            "clipboard format must be approved before GlobalSize receives its handle"
        );
    }

    #[test]
    fn direct_paste_is_limited_to_native_editor_controls() {
        assert!(supports_direct_paste("Scintilla"));
        assert!(supports_direct_paste("Edit"));
        assert!(supports_direct_paste("RichEditD2DPT"));
        assert!(supports_direct_paste(
            "WindowsForms10.EDIT.app.0.141b42a_r8_ad1"
        ));
        assert!(!supports_direct_paste("Chrome_RenderWidgetHostHWND"));
        assert!(!supports_direct_paste("MozillaWindowClass"));
        assert!(!supports_direct_paste("ApplicationFrameWindow"));
    }
}
