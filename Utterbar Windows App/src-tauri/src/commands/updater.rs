//! In-app updater.
//!
//! The user-facing flow is:
//!   1. On app launch (if `general.check_updates_automatically`), the frontend
//!      calls `check_update`. We GET `utterbar.com/api/updater/check?current=X.Y.Z`
//!      and return whatever the server says.
//!   2. If an update is available, the `UpdateToast` offers an Install button
//!      that calls `install_update`. We download the new installer (.exe) to
//!      the user's temp folder, verify its SHA-256 against the server's hash,
//!      spawn it as a detached process with NSIS `/S` (silent install), and
//!      exit the app so the installer can overwrite the .exe files. The NSIS
//!      installer's post-install step re-launches Utterbar, which gives the
//!      user the "close → update → relaunch" UX without any manual steps.
//!
//! Why not `tauri-plugin-updater`? We considered it. The plugin mandates a
//! minisign-signed `latest.json` hosted at a fixed path, plus a matching
//! public key embedded at build time. That machinery is fine for a mature
//! app but adds a new signing workflow Jay has to remember on every release
//! (generate/protect the minisign key, sign each bundle), and the
//! integration is load-bearing enough that a botched signing break locks
//! every user out of updates. Our release pipeline already signs nothing
//! beyond Windows code-signing (NSIS .exe); by doing our own SHA-256 verify
//! against the admin API's canonical hash we get "the right bits reached
//! the user" without a second signing flow. If we ever want Apple-style
//! background auto-apply, we can swap in the plugin later — the command
//! surface here matches the one the frontend already speaks.
//!
//! ## Server endpoint contract (to implement on utterbar.com)
//!
//! ```http
//! GET https://utterbar.com/api/updater/check?current=0.6.0&channel=stable
//! ```
//!
//! - `current` (required): the installed version's `CARGO_PKG_VERSION`.
//! - `channel` (optional, default `stable`): `stable` or `beta`.
//!
//! Response 200 JSON:
//! ```json
//! {
//!   "available": true,
//!   "version": "0.6.1",
//!   "download_url": "https://dl.utterbar.com/releases/0.6.1/Utterbar-Setup-0.6.1.exe",
//!   "sha256": "47bd8249…",
//!   "notes_markdown": "- First user-facing change\n- Second…",
//!   "size_bytes": 10962662
//! }
//! ```
//!
//! When no update is available, `{ "available": false, "current": "0.6.0" }`.
//! On server error, any non-200 is treated as "no update" by the client so a
//! broken endpoint never breaks the app's normal operation.

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::AppHandle;
use ts_rs::TS;

use crate::util::result::{CmdResult, OrliqError};

const DEFAULT_BASE_URL: &str = "https://utterbar.com";

/// Payload returned to the frontend. Matches the existing `UpdateInfo` shape
/// the `UpdateToast` consumes, extended with the fields we need to download
/// and verify. `notes` is the only field already displayed; everything else
/// is transport metadata.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct UpdateInfo {
    pub current: String,
    pub available: Option<String>,
    pub notes: Option<String>,
    /// Canonical HTTPS URL the installer will be downloaded from. None when
    /// no update is available. Kept visible to the UI so a "Download
    /// manually" fallback could link to it if `install_update` ever fails.
    pub download_url: Option<String>,
    /// Hex-encoded SHA-256 of the installer. We verify this before
    /// launching the downloaded binary — a surprise-mismatch never spawns.
    pub sha256: Option<String>,
    /// File size in bytes, for a hypothetical progress UI later.
    pub size_bytes: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct ServerAvailable {
    available: bool,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    download_url: Option<String>,
    #[serde(default)]
    sha256: Option<String>,
    #[serde(default)]
    notes_markdown: Option<String>,
    #[serde(default)]
    size_bytes: Option<u64>,
}

/// Ask the server what the latest version is for this channel.
///
/// Any transport or parse error folds into "no update available" rather
/// than surfacing an error to the UI — the toast shouldn't flash red just
/// because the update endpoint is unreachable at launch. That would be
/// noisy, user-hostile, and fires on every flaky hotel-wifi morning.
#[tauri::command]
pub async fn check_update() -> CmdResult<UpdateInfo> {
    let current = env!("CARGO_PKG_VERSION").to_string();
    let no_update = || UpdateInfo {
        current: current.clone(),
        available: None,
        notes: None,
        download_url: None,
        sha256: None,
        size_bytes: None,
    };

    let user_agent = format!("Utterbar/{} (windows; x86_64)", current);
    let client = match Client::builder()
        .user_agent(user_agent)
        .timeout(Duration::from_secs(8))
        .connect_timeout(Duration::from_secs(4))
        .build()
    {
        Ok(c) => c,
        Err(_) => return Ok(no_update()),
    };

    let base =
        std::env::var("UTTERBAR_LICENCE_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());
    let url = format!(
        "{}/api/updater/check?current={}&channel=stable",
        base, current
    );

    let resp = match client.get(&url).send().await {
        Ok(r) if r.status().is_success() => r,
        _ => return Ok(no_update()),
    };

    let payload: ServerAvailable = match resp.json().await {
        Ok(p) => p,
        Err(_) => return Ok(no_update()),
    };

    if !payload.available || payload.version.is_none() || payload.download_url.is_none() {
        return Ok(no_update());
    }

    Ok(UpdateInfo {
        current,
        available: payload.version,
        notes: payload.notes_markdown,
        download_url: payload.download_url,
        sha256: payload.sha256,
        size_bytes: payload.size_bytes,
    })
}

/// Download the new installer, verify its hash, spawn the silent installer,
/// and exit the running app.
///
/// Exit-after-spawn is deliberate: NSIS needs exclusive access to the .exe
/// files it's overwriting, which means the currently-running app MUST die
/// before the installer can make progress. If the user kept the window up,
/// they'd stare at a hung Installing… spinner that never completes. We spawn
/// the installer detached, then call `app.exit(0)` after a short delay so the
/// child has time to initialise. NSIS's post-install hook re-launches
/// Utterbar, closing the loop.
#[tauri::command]
pub async fn install_update(handle: AppHandle) -> CmdResult<()> {
    let info = check_update().await?;
    let (version, download_url, sha256) = match (info.available, info.download_url, info.sha256) {
        (Some(v), Some(url), Some(sha)) => (v, url, sha),
        _ => {
            return Err(OrliqError::Internal {
                message: "No update is available to install.".into(),
            });
        }
    };

    let user_agent = format!("Utterbar/{} (windows; x86_64)", env!("CARGO_PKG_VERSION"));
    let client = Client::builder()
        .user_agent(user_agent)
        .timeout(Duration::from_secs(120))
        .connect_timeout(Duration::from_secs(8))
        .build()
        .map_err(|e| OrliqError::Internal {
            message: format!("http client build: {e}"),
        })?;

    let bytes = client
        .get(&download_url)
        .send()
        .await
        .map_err(|e| OrliqError::Internal {
            message: format!("download failed: {e}"),
        })?
        .bytes()
        .await
        .map_err(|e| OrliqError::Internal {
            message: format!("download stream failed: {e}"),
        })?;

    // Hash-check before we spawn. A tampered-in-transit binary would be
    // catastrophic (the user launches whatever lands at this URL as elevated
    // installer); mismatch = bail loud.
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let got = format!("{:x}", hasher.finalize());
    if got.to_lowercase() != sha256.to_lowercase() {
        return Err(OrliqError::Internal {
            message: "Downloaded installer failed integrity check.".into(),
        });
    }

    // Write to temp. Filename includes the version so a left-over temp from
    // a previous update is obvious in %TEMP%, and a user running the file
    // manually still sees what version it is.
    let path: PathBuf = std::env::temp_dir().join(format!("Utterbar-Setup-{}.exe", version));
    {
        let mut f = File::create(&path).map_err(|e| OrliqError::Internal {
            message: format!("temp write: {e}"),
        })?;
        f.write_all(&bytes).map_err(|e| OrliqError::Internal {
            message: format!("temp write: {e}"),
        })?;
    }

    // Spawn the installer silently. `/S` is NSIS's silent-install flag — the
    // user sees no UI, the installer overwrites the app in place, and our
    // post-install hook in `installer.nsh` re-launches Utterbar.
    spawn_installer(&path)?;

    // Exit after a short beat so the spawn has stuck. Using `handle.exit(0)`
    // instead of `std::process::exit(0)` lets Tauri's cleanup (tray, window
    // hooks) run, so we don't leave a dangling tray icon while the installer
    // works.
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_millis(400)).await;
        handle.exit(0);
    });
    Ok(())
}

#[cfg(target_os = "windows")]
fn spawn_installer(path: &std::path::Path) -> Result<(), OrliqError> {
    use std::os::windows::process::CommandExt;
    const DETACHED_PROCESS: u32 = 0x0000_0008;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
    std::process::Command::new(path)
        .arg("/S")
        .creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP)
        .spawn()
        .map_err(|e| OrliqError::Internal {
            message: format!("spawn installer: {e}"),
        })?;
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn spawn_installer(_path: &std::path::Path) -> Result<(), OrliqError> {
    Err(OrliqError::Internal {
        message: "In-app updates are Windows-only at the moment.".into(),
    })
}
