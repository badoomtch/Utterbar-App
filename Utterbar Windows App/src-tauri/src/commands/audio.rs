use std::sync::Arc;

use tauri::{AppHandle, State};
use tokio::time::{sleep, Duration};

use crate::app_state::AppState;
use crate::domain::settings::DEFAULT_CAPTURE_TAIL_PAD_MS;
use crate::services::audio::{meter, InputDevice};
use crate::util::result::{CmdResult, OrliqError};

#[tauri::command]
pub async fn list_input_devices(state: State<'_, Arc<AppState>>) -> CmdResult<Vec<InputDevice>> {
    state.audio.devices().await.map_err(|e| OrliqError::Device {
        message: e.to_string(),
    })
}

#[tauri::command]
pub async fn start_meter(
    handle: AppHandle,
    state: State<'_, Arc<AppState>>,
    device_id: Option<String>,
) -> CmdResult<()> {
    let (capture, meta) = state
        .audio
        .start_capture(device_id, DEFAULT_CAPTURE_TAIL_PAD_MS)
        .await
        .map_err(|e| OrliqError::Device {
            message: e.to_string(),
        })?;
    if meta.fallback_reason.is_some() {
        tracing::warn!(
            resolved_device_name = %meta.resolved_device_name,
            reason = ?meta.fallback_reason,
            "start_meter: input-device fallback occurred"
        );
    }
    meter::spawn_pump(handle, capture);
    Ok(())
}

#[tauri::command]
pub async fn stop_meter(state: State<'_, Arc<AppState>>) -> CmdResult<()> {
    let _ = state.audio.stop_capture().await;
    Ok(())
}

#[tauri::command]
pub async fn record_test_clip(
    state: State<'_, Arc<AppState>>,
    device_id: Option<String>,
    seconds: u64,
) -> CmdResult<TestClip> {
    let _ = state
        .audio
        .start_capture(device_id, DEFAULT_CAPTURE_TAIL_PAD_MS)
        .await
        .map_err(|e| OrliqError::Device {
            message: e.to_string(),
        })?
        .0;
    sleep(Duration::from_secs(seconds.min(15))).await;
    let final_ = state.audio.stop_capture().await;
    Ok(match final_ {
        Some(f) => TestClip {
            sample_count: f.pcm.len() as u32,
            sample_rate: f.sample_rate,
            duration_ms: f.duration_ms as u32,
        },
        None => TestClip::default(),
    })
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct TestClip {
    pub sample_count: u32,
    pub sample_rate: u32,
    pub duration_ms: u32,
}
