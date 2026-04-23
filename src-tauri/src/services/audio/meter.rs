//! Pumps meter frames over the Tauri event bus at ~30 Hz.

use std::sync::Arc;

use tauri::{AppHandle, Emitter};

use crate::domain::{MeterFrame, EVT_DICTATION_METER};
use crate::services::audio::capture::CaptureHandle;
use crate::util::time::now_ms;

pub fn spawn_pump(handle: AppHandle, capture: Arc<CaptureHandle>) {
    tokio::spawn(async move {
        let mut rx = capture.meter_rx.lock().await;
        while let Some((rms, peak)) = rx.recv().await {
            let frame = MeterFrame { rms, peak, ts_ms: now_ms() };
            let _ = handle.emit(EVT_DICTATION_METER, frame);
        }
    });
}
