//! Dedicated OS thread that owns COM apartment state for UIA calls.
//!
//! Why a dedicated thread exists at all:
//!
//! UI Automation is fundamentally STA (single-threaded apartment) COM. Every
//! UIA object — IUIAutomation, IUIAutomationElement, IUIAutomationTextPattern,
//! IUIAutomationTextRangeArray, the BSTRs they return — is associated with
//! the apartment of the thread that created or first marshalled it. In an
//! STA, COM's threading rules require that interface methods be invoked on
//! the same thread that initialised the apartment, OR via a proxy marshalled
//! through the object's apartment.
//!
//! When UIA calls were made from tokio worker threads (via `stop_recording`
//! -> `context.snapshot()` -> `read_selection_uia`), several things could
//! go wrong:
//!
//!   * The worker thread might have been a "fresh" tokio thread that had
//!     never been CoInitialized. First call succeeds with `COINIT_APARTMENTTHREADED`
//!     (init count 1, STA). No CoUninitialize was ever called — leak.
//!   * The worker might have previously been used by
//!     `services/audio/loopback.rs` or `services/audio_ducking.rs`, both of
//!     which call `CoInitializeEx(COINIT_MULTITHREADED)`. When UIA then
//!     tries `CoInitializeEx(COINIT_APARTMENTTHREADED)` on the same thread,
//!     Windows returns `RPC_E_CHANGED_MODE` — the STA init fails, but the
//!     MTA init count is unchanged. UIA calls proceed under MTA semantics
//!     against an STA-designed API.
//!   * The `audio_ducking::ComGuard` always calls `CoUninitialize` in Drop
//!     without tracking whether its `CoInitializeEx` actually succeeded.
//!     If it ran on a thread already CoInitialized-into-STA by UIA, the
//!     Drop would over-release the STA init count, nuking UIA's apartment
//!     mid-use.
//!   * STA objects marshalled into the MTA without proxies are UB. UIA
//!     calls hitting ORT/DirectML-initialised threads touch each other's
//!     allocator state and eventually trip the heap integrity check:
//!     STATUS_HEAP_CORRUPTION (0xC0000374), closely followed by an
//!     illegal-instruction dialog when Windows tears down the process.
//!
//! The fix is architectural. This module spawns a single OS thread that:
//!
//!   1. calls `CoInitializeEx(COINIT_APARTMENTTHREADED)` exactly once,
//!   2. runs a message loop (not a Win32 message pump — our own `mpsc`
//!      receive loop is enough for STA requirements; COM is happy as long
//!      as all interface calls happen on this thread),
//!   3. receives requests via `crossbeam_channel`, executes them, returns
//!      the response via a one-shot reply channel,
//!   4. calls `CoUninitialize` on clean shutdown (never in practice —
//!      the thread lives for the lifetime of the app).
//!
//! Every UIA call from the rest of the codebase goes through `snapshot_selection`
//! which posts a request to this thread. Tokio workers never touch UIA
//! directly, so the COM mode mismatch is impossible.

use std::sync::{Mutex, OnceLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crossbeam_channel::{bounded, Sender};
use windows::core::Interface;
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
    COINIT_APARTMENTTHREADED,
};
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, IUIAutomationTextPattern, UIA_TextPatternId,
};

const MAX_SELECTION_CHARS: usize = 4_000;

/// Request for the UIA worker thread. Only one request shape exists today;
/// if we add more UIA-backed reads in future (e.g. active form field state)
/// they go here as new variants so the worker stays a single channel
/// consumer.
enum Request {
    ReadSelection {
        hwnd_value: isize,
        reply: Sender<Option<String>>,
    },
    Shutdown,
}

/// Handle onto the UIA thread. Single instance for the whole app, stored
/// in a OnceLock so the first caller spawns the thread and every later
/// caller reuses it.
struct UiaThread {
    tx: Sender<Request>,
    // Kept so the thread joins on app shutdown. Wrapped in a Mutex<Option>
    // so `shutdown()` can take it and join without moving the OnceLock.
    join: Mutex<Option<JoinHandle<()>>>,
}

static INSTANCE: OnceLock<UiaThread> = OnceLock::new();

fn get_or_spawn() -> &'static UiaThread {
    INSTANCE.get_or_init(|| {
        let (tx, rx) = bounded::<Request>(8);
        let join = thread::Builder::new()
            .name("utterbar-uia".into())
            .spawn(move || {
                // SAFETY: CoInitializeEx must be called exactly once on this
                // thread. We hold the apartment for the thread's entire
                // lifetime and never migrate. Return value may be S_OK
                // (apartment created) or S_FALSE (already initialised by
                // a prior call on this thread — impossible since this is
                // a fresh thread, but harmless). Any other HRESULT is a
                // hard fail and we log + exit the worker.
                let hr = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
                if hr.is_err() {
                    tracing::error!(?hr, "uia_thread: CoInitializeEx failed — UIA will be unavailable");
                    // Drain any queued requests with None so callers don't
                    // block forever on reply channels.
                    for req in rx.iter() {
                        if let Request::ReadSelection { reply, .. } = req {
                            let _ = reply.send(None);
                        }
                    }
                    return;
                }
                tracing::info!("uia_thread: COM initialised (STA), ready");

                for req in rx.iter() {
                    match req {
                        Request::ReadSelection { hwnd_value, reply } => {
                            let hwnd = HWND(hwnd_value as *mut std::ffi::c_void);
                            let result = read_selection_inner(hwnd);
                            let _ = reply.send(result);
                        }
                        Request::Shutdown => break,
                    }
                }

                // SAFETY: balances the CoInitializeEx above. The thread is
                // about to exit, so no further COM calls on this thread.
                unsafe { CoUninitialize() };
                tracing::info!("uia_thread: shut down cleanly");
            })
            .expect("failed to spawn utterbar-uia thread");

        UiaThread {
            tx,
            join: Mutex::new(Some(join)),
        }
    })
}

/// Read the currently-selected text from the foreground window, via the
/// dedicated UIA thread. Blocks the caller briefly (up to 200 ms) while
/// the worker processes — this is invoked from async contexts via
/// `spawn_blocking` so it doesn't stall the tokio runtime.
///
/// Returns `None` if no selection exists, if the UIA worker isn't running,
/// or if the worker times out. Callers must never interpret `None` as an
/// error — selection context is optional for every caller.
pub fn snapshot_selection(hwnd: HWND) -> Option<String> {
    let t = get_or_spawn();
    let (reply_tx, reply_rx) = bounded::<Option<String>>(1);
    if t.tx
        .send(Request::ReadSelection {
            hwnd_value: hwnd.0 as isize,
            reply: reply_tx,
        })
        .is_err()
    {
        tracing::warn!("uia_thread: send failed (worker gone?)");
        return None;
    }
    // 200 ms is a lot for UIA in the happy path (<5 ms) but covers the
    // pathological case where the foreground window's accessibility provider
    // is doing IO. Returning None on timeout is safe — the caller treats
    // missing selection as "no selection available" either way.
    match reply_rx.recv_timeout(Duration::from_millis(200)) {
        Ok(v) => v,
        Err(_) => {
            tracing::warn!("uia_thread: reply timeout reading selection");
            None
        }
    }
}

/// Shutdown hook. Called from the Tauri exit path to let the UIA thread
/// tear down its COM apartment cleanly before the process dies.
///
/// Not strictly required — the OS reclaims all resources on exit — but
/// matches the symmetry of `get_or_spawn` and makes the thread's
/// intentional lifetime explicit in the log.
#[allow(dead_code)]
pub fn shutdown() {
    if let Some(t) = INSTANCE.get() {
        let _ = t.tx.send(Request::Shutdown);
        if let Ok(mut slot) = t.join.lock() {
            if let Some(join) = slot.take() {
                let _ = join.join();
            }
        }
    }
}

/// The actual UIA call. Runs on the dedicated thread. Identical in spirit
/// to the old `read_selection_uia` but now with no CoInitializeEx inside —
/// the thread already owns its apartment.
fn read_selection_inner(hwnd: HWND) -> Option<String> {
    if hwnd.0.is_null() {
        return None;
    }
    unsafe {
        let automation: IUIAutomation =
            CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER).ok()?;

        let element = automation
            .GetFocusedElement()
            .ok()
            .or_else(|| automation.ElementFromHandle(hwnd).ok())?;

        let pattern_unk = element.GetCurrentPattern(UIA_TextPatternId).ok()?;
        let text_pattern: IUIAutomationTextPattern = pattern_unk.cast().ok()?;
        let selection = text_pattern.GetSelection().ok()?;
        let len = selection.Length().ok()?;
        if len == 0 {
            return None;
        }

        let mut out = String::new();
        for i in 0..len {
            let range = selection.GetElement(i).ok()?;
            let bstr = range.GetText(MAX_SELECTION_CHARS as i32).ok()?;
            out.push_str(&bstr.to_string());
            if out.chars().count() >= MAX_SELECTION_CHARS {
                break;
            }
        }
        let trimmed = out.trim();
        if trimmed.is_empty() {
            None
        } else if trimmed.chars().count() > MAX_SELECTION_CHARS {
            Some(
                trimmed
                    .chars()
                    .take(MAX_SELECTION_CHARS)
                    .collect::<String>()
                    + "…",
            )
        } else {
            Some(trimmed.to_string())
        }
    }
}
