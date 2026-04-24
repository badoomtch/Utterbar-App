//! Active-window + selection + clipboard readers for Windows.
//!
//! Selection reading (UIA) is delegated to a dedicated OS thread in
//! `super::uia_thread`. See that module for the full rationale — the
//! short version is that UIA is an STA-COM API and calling it from
//! tokio workers that may have previously been used by other COM
//! callers (WASAPI, audio ducking) was the root cause of the
//! STATUS_HEAP_CORRUPTION crashes observed through v0.6.3.

use windows::Win32::Foundation::{HWND, MAX_PATH};
use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
use windows::Win32::UI::WindowsAndMessaging::{
    GetClassNameW, GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW,
    GetWindowThreadProcessId,
};

use crate::services::context::ActiveAppContext;

pub fn snapshot(include_selection: bool, include_clipboard: bool) -> ActiveAppContext {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.0.is_null() {
        return ActiveAppContext::default();
    }

    let title = read_window_title(hwnd);
    let class = read_window_class(hwnd);
    let exe = read_window_exe(hwnd);

    // Route selection reads through the dedicated UIA thread. Blocks
    // the current (spawn_blocking) thread briefly while the UIA thread
    // services the request; must never be invoked from an async
    // context directly.
    let selection = if include_selection {
        super::uia_thread::snapshot_selection(hwnd)
    } else {
        None
    };
    let clipboard = if include_clipboard {
        read_clipboard()
    } else {
        None
    };

    ActiveAppContext {
        exe,
        title,
        class,
        selection,
        clipboard,
    }
}

fn read_window_title(hwnd: HWND) -> Option<String> {
    let len = unsafe { GetWindowTextLengthW(hwnd) };
    if len == 0 {
        return None;
    }
    let mut buf = vec![0u16; (len + 1) as usize];
    let copied = unsafe { GetWindowTextW(hwnd, &mut buf) };
    if copied <= 0 {
        return None;
    }
    Some(String::from_utf16_lossy(&buf[..copied as usize]))
}

fn read_window_class(hwnd: HWND) -> Option<String> {
    let mut buf = [0u16; 256];
    let n = unsafe { GetClassNameW(hwnd, &mut buf) };
    if n <= 0 {
        return None;
    }
    Some(String::from_utf16_lossy(&buf[..n as usize]))
}

fn read_window_exe(hwnd: HWND) -> Option<String> {
    unsafe {
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return None;
        }
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; MAX_PATH as usize];
        let n = GetModuleFileNameExW(handle, None, &mut buf);
        let _ = windows::Win32::Foundation::CloseHandle(handle);
        if n == 0 {
            return None;
        }
        let path = String::from_utf16_lossy(&buf[..n as usize]);
        path.split(['/', '\\']).next_back().map(|s| s.to_string())
    }
}

fn read_clipboard() -> Option<String> {
    match crate::platform::windows::text_inject::read_clipboard_text() {
        Ok(mut s) => {
            const MAX_CHARS: usize = 4_000;
            if s.chars().count() > MAX_CHARS {
                s = s.chars().take(MAX_CHARS).collect::<String>() + "…";
            }
            Some(s)
        }
        Err(_) => None,
    }
}
