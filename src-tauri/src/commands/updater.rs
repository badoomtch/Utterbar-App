//! Updater stub. Disabled until we have a real signing pubkey and a hosted update feed.
//! Leaving an empty module keeps the command surface stable so the frontend toast can
//! call into it once we turn the plugin back on.

use serde::Serialize;
use ts_rs::TS;

use crate::util::result::CmdResult;

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct UpdateInfo {
    pub current: String,
    pub available: Option<String>,
    pub notes: Option<String>,
}

#[tauri::command]
pub async fn check_update() -> CmdResult<UpdateInfo> {
    Ok(UpdateInfo {
        current: env!("CARGO_PKG_VERSION").into(),
        available: None,
        notes: None,
    })
}

#[tauri::command]
pub async fn install_update() -> CmdResult<()> {
    Ok(())
}
