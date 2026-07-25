use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use ts_rs::TS;

use crate::app_state::AppState;
use crate::domain::{Replacement, VocabEntry, VocabularyImport};
use crate::util::result::{CmdResult, OrliqError};

#[tauri::command]
pub async fn list_vocabulary(state: State<'_, Arc<AppState>>) -> CmdResult<Vec<VocabEntry>> {
    state.vocabulary.list_words().map_err(OrliqError::from)
}

#[tauri::command]
pub async fn upsert_vocabulary(
    state: State<'_, Arc<AppState>>,
    entry: VocabEntry,
) -> CmdResult<VocabEntry> {
    state
        .vocabulary
        .upsert_word(entry)
        .map_err(OrliqError::from)
}

#[tauri::command]
pub async fn delete_vocabulary(state: State<'_, Arc<AppState>>, id: String) -> CmdResult<()> {
    state.vocabulary.delete_word(&id).map_err(OrliqError::from)
}

#[tauri::command]
pub async fn list_replacements(state: State<'_, Arc<AppState>>) -> CmdResult<Vec<Replacement>> {
    state
        .vocabulary
        .list_replacements()
        .map_err(OrliqError::from)
}

#[tauri::command]
pub async fn upsert_replacement(
    state: State<'_, Arc<AppState>>,
    r: Replacement,
) -> CmdResult<Replacement> {
    state
        .vocabulary
        .upsert_replacement(r)
        .map_err(OrliqError::from)
}

#[tauri::command]
pub async fn delete_replacement(state: State<'_, Arc<AppState>>, id: String) -> CmdResult<()> {
    state
        .vocabulary
        .delete_replacement(&id)
        .map_err(OrliqError::from)
}

#[tauri::command]
pub async fn import_vocabulary(
    state: State<'_, Arc<AppState>>,
    payload: VocabularyImport,
) -> CmdResult<()> {
    for w in payload.words {
        let _ = state.vocabulary.upsert_word(w);
    }
    for r in payload.replacements {
        let _ = state.vocabulary.upsert_replacement(r);
    }
    Ok(())
}

#[tauri::command]
pub async fn export_vocabulary(state: State<'_, Arc<AppState>>) -> CmdResult<VocabularyImport> {
    Ok(VocabularyImport {
        words: state.vocabulary.list_words().map_err(OrliqError::from)?,
        replacements: state
            .vocabulary
            .list_replacements()
            .map_err(OrliqError::from)?,
    })
}

// -----------------------------------------------------------------------------
// Quick add-to-vocabulary capture path.
//
// Triggered by the AddToVocabulary global shortcut OR by an explicit user
// click on the Vocabulary page's "Capture selection" button. Workflow:
//
//   1. Read the current Windows plain-text clipboard (`CF_UNICODETEXT`).
//      This is what we will restore at the end — we deliberately do NOT
//      try to preserve images, file lists, RTF, or HTML. The honest
//      trade-off: a user who had a screenshot or a copied file in their
//      clipboard and triggers add-to-vocabulary will lose that clipboard
//      content. The alternative (full multi-format save+restore) is a
//      lot of code for marginal real-world value, since the workflow is
//      "I have a word selected, capture it" — and in that mental model
//      the clipboard is incidental. Documented here, in the JS layer's
//      QuickAddWindow info text, and in the docs.
//
//   2. Synthesise Ctrl+C via SendInput. The foreground app handles it
//      and writes the user's selected text to the clipboard.
//
//   3. Wait ~120 ms for the clipboard write to land. Some apps are
//      slower; the wait could be tuned per-app but a flat 120 ms covers
//      the apps users actually use (browsers, editors, Slack, Office)
//      without making the workflow feel sluggish.
//
//   4. Read the clipboard again. If it's identical to the pre-capture
//      content, the user had nothing selected — open the window with an
//      empty input and a hint instead of failing silently.
//
//   5. Sanitise: trim, replace newlines with spaces, collapse runs of
//      whitespace, reject if longer than 60 chars or 5 words. This is a
//      vocabulary entry workflow, not a paragraph-capture workflow; we
//      keep it tight and explicit.
//
//   6. Restore the original plain-text clipboard so we don't leave the
//      user's clipboard mutated by our capture.
//
//   7. Show the vocab-add window (created lazily on first call) with
//      the captured text emitted as an event payload the React side
//      consumes on mount.

/// Public payload emitted to the vocab-add window when capture completes.
/// Matches the TS shape `VocabCapturePayload`.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct VocabCapturePayload {
    /// The captured + sanitised text. Empty when nothing was selected;
    /// the React layer treats empty as "show empty input + hint".
    pub captured: String,
    /// True when the original selection exceeded our length limits and
    /// was rejected. The React layer renders an explanatory message
    /// instead of dropping a silent empty-input.
    pub rejected_too_long: bool,
}

const VOCAB_CAPTURE_EVT: &str = "vocab:capture";
const VOCAB_ADD_LABEL: &str = "vocab-add";

#[tauri::command]
pub async fn open_vocab_quick_add(handle: AppHandle) -> CmdResult<()> {
    capture_and_open(handle).await
}

/// Entry point used by the global-shortcut dispatcher. Same code path as the
/// manual command — the shortcut handler in the frontend invokes
/// `open_vocab_quick_add` directly, but exposing this as a dedicated entry
/// keeps the semantics clear.
#[allow(dead_code)]
pub async fn handle_shortcut(handle: AppHandle) {
    if let Err(e) = capture_and_open(handle).await {
        tracing::warn!(?e, "vocab quick-add capture failed");
    }
}

async fn capture_and_open(handle: AppHandle) -> CmdResult<()> {
    let captured = match capture_selection_via_ctrl_c() {
        Ok(text) => text,
        Err(e) => {
            tracing::warn!(?e, "selection capture failed; opening empty window");
            String::new()
        }
    };

    // Sanitise. Keep this small + explicit — no regex, no Unicode
    // normalisation surprises. trim → newline-to-space → collapse-runs.
    let cleaned = sanitise(&captured);
    let rejected_too_long = !cleaned.is_empty() && exceeds_limits(&cleaned);
    let final_text = if rejected_too_long {
        String::new()
    } else {
        cleaned
    };

    // Open / focus the vocab-add window. Lazy create — keeps the cold-
    // start cost down for the 99% of sessions that don't trigger it.
    show_vocab_add_window(&handle)?;

    // Emit the payload AFTER show() so the listener is mounted.
    let payload = VocabCapturePayload {
        captured: final_text,
        rejected_too_long,
    };
    let _ = handle.emit(VOCAB_CAPTURE_EVT, payload);
    Ok(())
}

fn sanitise(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    // Replace any newline / carriage return with a single space, then
    // collapse runs of whitespace into a single space. We accept a
    // multi-word entry (e.g. "Liam Parry") so we don't strip spaces.
    let mut out = String::with_capacity(trimmed.len());
    let mut prev_was_space = false;
    for ch in trimmed.chars() {
        if ch == '\n' || ch == '\r' || ch == '\t' || ch == ' ' {
            if !prev_was_space {
                out.push(' ');
                prev_was_space = true;
            }
        } else {
            out.push(ch);
            prev_was_space = false;
        }
    }
    out
}

fn exceeds_limits(s: &str) -> bool {
    // 60 chars / 5 words. Vocabulary entries are names + technical
    // terms, not paragraphs — anything longer is almost certainly a
    // mis-captured selection or an accidental Ctrl+A first.
    s.chars().count() > 60 || s.split_whitespace().count() > 5
}

/// Show or create the vocab-add window. Always-on-top, fixed size,
/// centred on the current monitor. The caller is responsible for
/// emitting the capture payload after this returns.
fn show_vocab_add_window(handle: &AppHandle) -> CmdResult<()> {
    if let Some(w) = handle.get_webview_window(VOCAB_ADD_LABEL) {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
        return Ok(());
    }
    // First-call path: build and show. The webview URL points the route
    // at /vocab-add — a separate React route mounted in main.tsx. Always
    // on top so it stays visible while the user types.
    let builder = tauri::WebviewWindowBuilder::new(
        handle,
        VOCAB_ADD_LABEL,
        tauri::WebviewUrl::App("index.html#/vocab-add".into()),
    )
    .title("Add to Utterbar vocabulary")
    .inner_size(360.0, 220.0)
    .min_inner_size(300.0, 180.0)
    .resizable(false)
    .maximizable(false)
    .minimizable(false)
    .always_on_top(true)
    .center()
    .focused(true)
    .skip_taskbar(true)
    .decorations(true)
    .visible(true);

    let _w = builder.build().map_err(|e| OrliqError::Internal {
        message: format!("Could not open the quick-add window: {e}"),
    })?;
    Ok(())
}

// ----- Windows clipboard + SendInput plumbing ---------------------------

#[cfg(windows)]
fn capture_selection_via_ctrl_c() -> Result<String, OrliqError> {
    use std::thread::sleep;
    use std::time::Duration;

    let pre = read_clipboard_text().unwrap_or_default();
    synthesise_ctrl_c().map_err(|e| OrliqError::Internal {
        message: format!("Could not synthesise Ctrl+C: {e}"),
    })?;
    sleep(Duration::from_millis(120));
    let post = read_clipboard_text().unwrap_or_default();

    // Restore — even if read failed, write back whatever we had pre.
    if !pre.is_empty() {
        let _ = write_clipboard_text(&pre);
    }

    // If the clipboard didn't change, the user had nothing selected.
    // Treat that as "empty captured" rather than an error.
    if post == pre {
        return Ok(String::new());
    }
    Ok(post)
}

#[cfg(not(windows))]
fn capture_selection_via_ctrl_c() -> Result<String, OrliqError> {
    Err(OrliqError::Internal {
        message: "Quick add-to-vocabulary capture is only available on Windows.".into(),
    })
}

// CF_UNICODETEXT is a Win32 clipboard format constant. Hardcoded to avoid
// adding the Win32_System_Ole feature flag for one number.
#[cfg(windows)]
const CF_UNICODETEXT: u32 = 13;

#[cfg(windows)]
fn read_clipboard_text() -> anyhow::Result<String> {
    use std::ptr::null_mut;
    use windows::Win32::Foundation::{HANDLE, HGLOBAL, HWND};
    use windows::Win32::System::DataExchange::{CloseClipboard, GetClipboardData, OpenClipboard};
    use windows::Win32::System::Memory::{GlobalLock, GlobalUnlock};

    unsafe {
        if OpenClipboard(HWND(null_mut())).is_err() {
            anyhow::bail!("OpenClipboard failed");
        }
        let result = (|| -> anyhow::Result<String> {
            let h: HANDLE = GetClipboardData(CF_UNICODETEXT)
                .map_err(|e| anyhow::anyhow!("GetClipboardData: {e}"))?;
            let hg = HGLOBAL(h.0);
            let p = GlobalLock(hg) as *const u16;
            if p.is_null() {
                anyhow::bail!("GlobalLock returned null");
            }
            // Find NUL terminator.
            let mut len = 0usize;
            while *p.add(len) != 0 {
                len += 1;
                if len > 1_048_576 {
                    // 1 MiB hard ceiling; defensive — we never read beyond
                    // this. A vocabulary capture won't be anywhere near.
                    break;
                }
            }
            let slice = std::slice::from_raw_parts(p, len);
            let s = String::from_utf16_lossy(slice);
            let _ = GlobalUnlock(hg);
            Ok(s)
        })();
        let _ = CloseClipboard();
        result
    }
}

#[cfg(windows)]
fn write_clipboard_text(text: &str) -> anyhow::Result<()> {
    use std::ptr::null_mut;
    use windows::Win32::Foundation::{HANDLE, HWND};
    use windows::Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
    };
    use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};

    let mut wide: Vec<u16> = text.encode_utf16().collect();
    wide.push(0);
    let bytes = wide.len() * std::mem::size_of::<u16>();
    unsafe {
        if OpenClipboard(HWND(null_mut())).is_err() {
            anyhow::bail!("OpenClipboard failed");
        }
        let result = (|| -> anyhow::Result<()> {
            let _ = EmptyClipboard();
            let hg = GlobalAlloc(GMEM_MOVEABLE, bytes)
                .map_err(|e| anyhow::anyhow!("GlobalAlloc: {e}"))?;
            let p = GlobalLock(hg) as *mut u16;
            if p.is_null() {
                anyhow::bail!("GlobalLock returned null");
            }
            std::ptr::copy_nonoverlapping(wide.as_ptr(), p, wide.len());
            let _ = GlobalUnlock(hg);
            // Pass ownership of hg to the clipboard. After this call, do
            // not free hg.
            let h = HANDLE(hg.0);
            SetClipboardData(CF_UNICODETEXT, h)
                .map_err(|e| anyhow::anyhow!("SetClipboardData: {e}"))?;
            Ok(())
        })();
        let _ = CloseClipboard();
        result
    }
}

#[cfg(windows)]
fn synthesise_ctrl_c() -> anyhow::Result<()> {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VIRTUAL_KEY, VK_C,
        VK_CONTROL,
    };

    fn key(vk: VIRTUAL_KEY, up: bool) -> INPUT {
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: 0,
                    dwFlags: if up {
                        KEYEVENTF_KEYUP
                    } else {
                        Default::default()
                    },
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }

    let inputs = [
        key(VK_CONTROL, false),
        key(VK_C, false),
        key(VK_C, true),
        key(VK_CONTROL, true),
    ];
    unsafe {
        let sent = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
        if sent != inputs.len() as u32 {
            anyhow::bail!("SendInput sent {} of {} events", sent, inputs.len());
        }
    }
    Ok(())
}
