//! Windows context service: active app, window title, class, selected text, clipboard.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct ActiveAppContext {
    pub exe: Option<String>,
    pub title: Option<String>,
    pub class: Option<String>,
    pub selection: Option<String>,
    pub clipboard: Option<String>,
}

pub struct ContextService;

impl ContextService {
    pub fn new() -> Self {
        Self
    }

    #[cfg(windows)]
    pub fn snapshot(&self, include_selection: bool, include_clipboard: bool) -> ActiveAppContext {
        crate::platform::windows::context::snapshot(include_selection, include_clipboard)
    }

    #[cfg(not(windows))]
    pub fn snapshot(&self, _include_selection: bool, _include_clipboard: bool) -> ActiveAppContext {
        ActiveAppContext::default()
    }
}
