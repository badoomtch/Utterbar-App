//! Active-window + selection + clipboard readers for Windows.

use windows::core::Interface;
use windows::Win32::Foundation::{HWND, MAX_PATH};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
};
use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, IUIAutomationTextPattern, UIA_TextPatternId,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetClassNameW, GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW,
    GetWindowThreadProcessId,
};

use crate::services::context::ActiveAppContext;

const MAX_SELECTION_CHARS: usize = 4_000;

pub fn snapshot(include_selection: bool, include_clipboard: bool) -> ActiveAppContext {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.0.is_null() {
        return ActiveAppContext::default();
    }

    let title = read_window_title(hwnd);
    let class = read_window_class(hwnd);
    let exe = read_window_exe(hwnd);

    let selection = if include_selection { read_selection_uia(hwnd) } else { None };
    let clipboard = if include_clipboard { read_clipboard() } else { None };

    ActiveAppContext { exe, title, class, selection, clipboard }
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

/// Walk the UIA tree under the foreground window, looking for a focused element that
/// exposes `TextPattern`. When found, read its currently selected range(s) and return
/// their concatenated text. Returns None when no selection can be retrieved — callers
/// must NEVER treat that as an error; it just means we don't have a selection to offer.
fn read_selection_uia(hwnd: HWND) -> Option<String> {
    unsafe {
        // Each call initialises its own STA; safe to call repeatedly.
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

        let automation: IUIAutomation =
            CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER).ok()?;

        // Prefer the focused element (more accurate than the root); fall back to the
        // element at the foreground window handle.
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
            Some(trimmed.chars().take(MAX_SELECTION_CHARS).collect::<String>() + "…")
        } else {
            Some(trimmed.to_string())
        }
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
