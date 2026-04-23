//! Read-only diagnostics surfaced in Settings → Advanced. Everything here is local;
//! nothing is phoned home.

use serde::Serialize;
use ts_rs::TS;

use crate::services::gpu::{self, GpuProbe};
use crate::util::paths;
use crate::util::result::{CmdResult, OrliqError};

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct Diagnostics {
    pub version: String,
    pub os: String,
    pub cpu_count: usize,
    pub log_path: String,
    pub db_path: String,
    pub models_path: String,
    /// Whether the Parakeet ONNX backend is compiled in. Kept as a boolean
    /// rather than dropped because the Advanced diagnostics panel surfaces it,
    /// and because earlier dev builds shipped it as a Cargo feature. The
    /// backend is now mandatory (sherpa-rs is a hard dependency), so this is
    /// always `true` on a production build — the field stays so older
    /// frontends keep deserialising the payload unchanged.
    pub onnx_feature: bool,
    pub llama_feature: bool,
}

#[tauri::command]
pub async fn get_diagnostics() -> CmdResult<Diagnostics> {
    Ok(Diagnostics {
        version: env!("CARGO_PKG_VERSION").into(),
        os: format!("{} {}", std::env::consts::OS, std::env::consts::ARCH),
        cpu_count: num_cpus(),
        log_path: paths::logs_dir()?.to_string_lossy().into(),
        // `orliq.db` is the on-disk filename committed in v0.x installs —
        // renaming would orphan every user's history + modes. Left as-is.
        db_path: paths::db_dir()?.join("orliq.db").to_string_lossy().into(),
        models_path: paths::models_dir()?.to_string_lossy().into(),
        onnx_feature: true,
        llama_feature: cfg!(feature = "llama"),
    })
}

/// Probe the local machine for GPU adapters via DXGI. Used by onboarding and the
/// Advanced settings pane to decide whether GPU acceleration is on the table.
#[tauri::command]
pub async fn probe_gpu() -> CmdResult<GpuProbe> {
    Ok(gpu::probe())
}

#[tauri::command]
pub async fn open_path(path: String) -> CmdResult<()> {
    opener_open(&path).map_err(|e| OrliqError::Io { message: e })
}

fn num_cpus() -> usize {
    std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1)
}

/// Tiny opener wrapper — uses `cmd /c start` on Windows, `open` on macOS, `xdg-open` on Linux.
fn opener_open(path: &str) -> Result<(), String> {
    #[cfg(windows)]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", path])
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}
