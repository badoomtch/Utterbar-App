//! Text output service: paste via Ctrl+V with keystroke-simulation fallback.

use anyhow::Result;

pub struct TextInjectService;

#[derive(Debug, Clone, Copy)]
pub enum Strategy {
    /// Paste via clipboard + Ctrl+V. When `fallback_to_keystrokes` is true and the paste
    /// appears to have failed (the clipboard still reads our text a moment later, which
    /// in virtually every app means the host ignored Ctrl+V), retry via per-character
    /// SendInput so apps that block synthetic Ctrl+V (RDP hosts, some Electron surfaces,
    /// certain games in focus) still receive the text.
    Paste {
        restore_clipboard: bool,
        fallback_to_keystrokes: bool,
    },
    Keystrokes,
}

impl TextInjectService {
    pub fn new() -> Self { Self }

    #[cfg(windows)]
    pub fn deliver(&self, text: &str, strategy: Strategy) -> Result<()> {
        crate::platform::windows::text_inject::deliver(text, strategy)
    }

    #[cfg(not(windows))]
    pub fn deliver(&self, _text: &str, _strategy: Strategy) -> Result<()> {
        Ok(())
    }

    /// Fire a single Enter keypress after delivery so messaging apps (Teams, Slack,
    /// Discord, anything with a "press Enter to send" input) submit the dictation
    /// without a second hotkey. Wired to Hold-Shift-to-auto-send: we only call it when
    /// the user was holding Shift at stop_recording.
    #[cfg(windows)]
    pub fn send_enter(&self) -> Result<()> {
        crate::platform::windows::text_inject::send_enter()
    }

    #[cfg(not(windows))]
    pub fn send_enter(&self) -> Result<()> {
        Ok(())
    }

    /// Non-blocking probe for whether Shift is currently held. Used by the Shift-to-auto
    /// -send feature at the exact moment stop_recording fires.
    #[cfg(windows)]
    pub fn shift_is_held(&self) -> bool {
        crate::platform::windows::text_inject::shift_is_held()
    }

    #[cfg(not(windows))]
    pub fn shift_is_held(&self) -> bool {
        false
    }
}
