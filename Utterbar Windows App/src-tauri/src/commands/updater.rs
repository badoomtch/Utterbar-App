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

use base64::Engine;
use ed25519_dalek::{Signature, VerifyingKey};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::AppHandle;
use ts_rs::TS;

use crate::util::result::{CmdResult, OrliqError};

const DEFAULT_BASE_URL: &str = "https://utterbar.com";

/// Ed25519 public key used to verify update metadata signatures.
///
/// **Operator-local key** — the matching private key never leaves the
/// release operator's machine. The publish script signs the canonical
/// metadata blob (see `canonical_metadata_blob`) with the private half;
/// this constant is what we verify against on every install. Replace
/// these zero bytes with the output of `php artisan updater:keygen` on
/// the operator's machine BEFORE shipping a build that depends on
/// signature verification — a build with all-zeros here will refuse
/// every update because no valid signature can verify against the
/// identity public key.
///
/// Distinct from the licence keypair (services/licence/verify.rs)
/// on purpose: a leak of either key does not compromise the other.
pub const UPDATER_PUBLIC_KEY: [u8; 32] = [
    0x3e, 0x15, 0xc0, 0x75, 0x7b, 0xcf, 0x7c, 0xec, 0x05, 0x9b, 0xd1, 0x97, 0x81, 0xd8, 0xa3, 0x51,
    0xeb, 0x6b, 0x51, 0x65, 0x22, 0xe2, 0x64, 0x85, 0x57, 0xb5, 0x52, 0xeb, 0xa0, 0xd6, 0x02, 0x87,
];

/// Magic line at the top of the canonical metadata blob — bumped if we
/// ever need to change the format. The server-side and operator-side
/// signers use the same string. See `canonical_metadata_blob`.
const UPDATE_METADATA_MAGIC: &str = "utterbar-update-v1";

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
    /// File size in bytes, used both for a future progress UI and as part
    /// of the signed metadata blob (any size mismatch voids the signature).
    pub size_bytes: Option<u64>,
    /// Base64-encoded Ed25519 signature over the canonical metadata blob.
    /// Verified against `UPDATER_PUBLIC_KEY` before any download starts.
    /// Kept on `UpdateInfo` rather than hidden inside `install_update` so
    /// the UI can surface "this release is unsigned" diagnostics later.
    pub signature: Option<String>,
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
    #[serde(default)]
    signature: Option<String>,
}

/// Ask the server what the latest version is for this channel.
///
/// Any transport or parse error folds into "no update available" rather
/// than surfacing an error to the UI — the toast shouldn't flash red just
/// because the update endpoint is unreachable at launch. That would be
/// noisy, user-hostile, and fires on every flaky hotel-wifi morning.
#[tauri::command]
pub async fn check_update(
    state: tauri::State<'_, std::sync::Arc<crate::app_state::AppState>>,
) -> CmdResult<UpdateInfo> {
    let current = env!("CARGO_PKG_VERSION").to_string();
    let no_update = || UpdateInfo {
        current: current.clone(),
        available: None,
        notes: None,
        download_url: None,
        sha256: None,
        size_bytes: None,
        signature: None,
    };

    // Read the channel preference from settings. Anything other than
    // "beta" is sent as "stable" server-side. Default is stable.
    let snapshot = state.settings.snapshot().await;
    let channel = match snapshot.advanced.update_channel.as_str() {
        "beta" => "beta",
        _ => "stable",
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
        "{}/api/updater/check?current={}&channel={}",
        base, current, channel
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
        signature: payload.signature,
    })
}

/// Download the new installer, verify its hash, spawn the silent installer,
/// queue a detached watcher that re-launches Utterbar after the install
/// completes, and exit the running app.
///
/// Why a detached watcher and not the NSIS post-install hook:
///   - NSIS in `currentUser` silent-install mode does not reliably run a
///     hook that exec's the freshly-installed binary. The hook fires inside
///     the installer's own process which then exits; whether the spawned
///     child outlives that exit depends on flags Tauri's installer template
///     doesn't expose. Empirically (v0.7.x): users see install succeed, no
///     relaunch, manual open required.
///   - We own the running process, we know the install target path, and we
///     can spawn an OS-level detached `cmd.exe` chain that outlives our own
///     exit. That is the only point in the chain where we can guarantee
///     the relaunch happens.
///
/// Sequence:
///   1. download + SHA-256 verify
///   2. write `pending_relaunch.json` to config_dir, marking "we asked the
///      installer to upgrade us — when you start, we expect to be on
///      version vX.Y.Z; if so, surface the post-update toast"
///   3. spawn the relaunch watcher: a detached `cmd /c ping -n 9 ... &
///      start "" "<install-path>"` chain that waits ~8 seconds (long
///      enough for NSIS to finish a 10MB install) then launches the
///      installed binary
///   4. spawn NSIS silently
///   5. handle.exit(0) after a short beat so the spawn settles
///
/// `ping -n 9 127.0.0.1` is the standard NSIS-friendly wait pattern on
/// Windows — `timeout` requires a console attached, `ping` doesn't.
#[tauri::command]
pub async fn install_update(
    handle: AppHandle,
    state: tauri::State<'_, std::sync::Arc<crate::app_state::AppState>>,
) -> CmdResult<()> {
    // Re-read the channel preference here so the signature check uses
    // exactly the channel string the operator signed. We don't echo
    // channel back on the wire — instead, the client requested it via
    // ?channel=, so the server's response IS for that channel. Asking
    // settings again is cheap and keeps the verification single-shot.
    let snapshot = state.settings.snapshot().await;
    let channel = match snapshot.advanced.update_channel.as_str() {
        "beta" => "beta",
        _ => "stable",
    };

    // Clone the inner Arc<AppState> before we hand `state` off to
    // `check_update` (which consumes it). The clone is what we use
    // for the `updater_install_in_progress` flag operations later.
    let state_for_flag = std::sync::Arc::clone(&state);

    let info = check_update(state).await?;
    let (version, download_url, sha256, size_bytes, signature) = match (
        info.available,
        info.download_url,
        info.sha256,
        info.size_bytes,
        info.signature,
    ) {
        (Some(v), Some(url), Some(sha), Some(size), Some(sig)) => (v, url, sha, size, sig),
        _ => {
            return Err(OrliqError::Internal {
                message: "No installable update available (missing metadata or signature).".into(),
            });
        }
    };

    let sig_ok = verify_update_signature(
        &signature,
        &version,
        channel,
        size_bytes,
        &sha256,
        &download_url,
    );
    if !sig_ok {
        return Err(OrliqError::Internal {
            message: "Update metadata failed Ed25519 signature verification — refusing to install."
                .into(),
        });
    }

    // Downgrade prevention. The server-side comparator already refuses
    // to advertise a non-newer version, but that's enforced server-side
    // only — a compromised server (or one returning manipulated JSON)
    // could push an older signed binary to clients to pivot through a
    // known-fixed bug in the older code. Compare here, refuse to
    // install anything <= the running version.
    let current_version = env!("CARGO_PKG_VERSION");
    if !semver_strictly_greater(&version, current_version) {
        return Err(OrliqError::Internal {
            message: format!(
                "Update server offered v{} which is not newer than v{}; refusing to install.",
                version, current_version
            ),
        });
    }

    // Pin the download host. The signature already commits to the URL,
    // but a defence-in-depth check here means even a botched signing
    // workflow (e.g. operator accidentally signing an attacker URL)
    // can never reach our spawn step.
    if !download_url.starts_with("https://dl.utterbar.com/") {
        return Err(OrliqError::Internal {
            message: "Update download URL is not on dl.utterbar.com; refusing to install."
                .to_string(),
        });
    }

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

    // Capture the running exe path BEFORE we touch anything — NSIS will
    // replace the file in place, but the path stays the same. The watcher
    // Mark "we expect to be on version `version` after the next launch". The
    // freshly-launched process reads this file from `config_dir` to decide
    // whether to surface the post-update toast.
    write_pending_relaunch(&version);

    // Mark the updater install as in-flight BEFORE we spawn the
    // installer or schedule the exit task. The main-window
    // CloseRequested handler in main.rs reads this flag (with
    // `Ordering::SeqCst` against this store) and, when set, skips its
    // `close_to_tray ? prevent_close + hide` intercept — without this
    // gate, `handle.exit(0)` below fires CloseRequested, the close
    // handler hides the window instead of letting the exit proceed,
    // and the running .exe stays alive while NSIS tries to overwrite
    // it. Net effect for the user: clicking Install & restart hid the
    // app to the tray and the install never applied. See AppState
    // field doc for the full chain.
    //
    // Set this AFTER all the can-fail prep (download, hash, sig
    // verify, downgrade-prevention, write to %TEMP%, write
    // pending_relaunch.json) so a `?` early-return never leaves the
    // flag stuck `true`. If `spawn_installer` itself fails below, the
    // explicit clear on the error branch resets the flag so the
    // user's next genuine X-click still goes to the tray as expected.
    use std::sync::atomic::Ordering;
    state_for_flag
        .updater_install_in_progress
        .store(true, Ordering::SeqCst);

    // Spawn the installer silently. `/S` is NSIS's silent-install flag —
    // the user sees no UI, the installer overwrites the app in place,
    // and the NSIS_HOOK_POSTINSTALL hook (in installer.nsh) re-launches
    // utterbar.exe via ExecShell once the install completes. The
    // PowerShell watcher we tried in v0.7.5–v0.8.3 is gone: it kept
    // failing because it ran outside the install process and had to
    // guess when the installer was done. ExecShell from inside NSIS is
    // how Squirrel / VS Code / Slack / Discord do it — the relaunch is
    // a child of the installer, fully detached, no shell scripting.
    if let Err(e) = spawn_installer(&path) {
        // Clear the intent flag so a failed spawn doesn't lock the
        // user out of normal close-to-tray on their next [X] click.
        state_for_flag
            .updater_install_in_progress
            .store(false, Ordering::SeqCst);
        return Err(e);
    }

    // Exit after a short beat so both spawns have stuck. Using
    // `handle.exit(0)` instead of `std::process::exit(0)` lets Tauri's
    // cleanup (tray, window hooks) run, so we don't leave a dangling tray
    // icon while the installer works.
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_millis(400)).await;
        handle.exit(0);
    });
    Ok(())
}

/// On-disk shape for the relaunch marker. Read on the next launch by
/// `consume_pending_relaunch` to surface the "you've been updated" toast.
#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct PendingRelaunch {
    /// The version we expect the freshly-installed binary to advertise via
    /// `CARGO_PKG_VERSION`. If it doesn't match (e.g. the install failed
    /// and the watcher relaunched the old binary, or the user reverted),
    /// we silently delete the marker without firing the toast.
    target_version: String,
    /// Wall-clock at the moment we wrote the marker. Purely for human-
    /// readable diagnostics in the log.
    written_at_unix: u64,
}

fn pending_relaunch_path() -> Option<PathBuf> {
    crate::util::paths::config_dir()
        .ok()
        .map(|d| d.join("pending_relaunch.json"))
}

fn write_pending_relaunch(target_version: &str) {
    let Some(path) = pending_relaunch_path() else {
        tracing::warn!("could not resolve config dir for pending_relaunch.json");
        return;
    };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let rec = PendingRelaunch {
        target_version: target_version.to_string(),
        written_at_unix: now,
    };
    let json = match serde_json::to_string(&rec) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(?e, "serialise pending_relaunch failed");
            return;
        }
    };
    match std::fs::write(&path, json) {
        Ok(()) => tracing::info!(?path, "wrote pending_relaunch marker"),
        Err(e) => tracing::warn!(?e, ?path, "failed to write pending_relaunch marker"),
    }
}

/// IPC entry. The frontend calls this once on mount; if we returned
/// Some, it fires the "Updated to vX.Y.Z" toast. The marker is deleted
/// on read regardless of return value, so re-mounts (e.g. dev HMR)
/// don't re-fire the toast.
#[tauri::command]
pub async fn consume_post_update_marker() -> CmdResult<Option<String>> {
    Ok(consume_pending_relaunch())
}

/// Non-consuming peek used by the startup hook to decide whether the
/// freshly-relaunched process should force-show the main window
/// (overriding `start_minimised`). Returns true only when the marker
/// exists AND its target_version matches the running binary, i.e.
/// the install genuinely upgraded us. Any mismatch returns false and
/// leaves the marker for `consume_pending_relaunch` to clean up.
///
/// Why a peek rather than reusing `consume_pending_relaunch`: the
/// existing consumer is owned by the frontend toast, which fires the
/// "Updated to vX.Y.Z" notification. If the startup hook consumed the
/// marker first, the toast would silently disappear on the very
/// launch where the user expects it. Peek-then-consume keeps both
/// contracts intact.
pub fn peek_pending_relaunch_for_force_show() -> bool {
    let Some(path) = pending_relaunch_path() else {
        return false;
    };
    if !path.exists() {
        return false;
    }
    let Ok(json) = std::fs::read_to_string(&path) else {
        return false;
    };
    let Ok(rec) = serde_json::from_str::<PendingRelaunch>(&json) else {
        return false;
    };
    let current = env!("CARGO_PKG_VERSION");
    let matched = rec.target_version == current;
    if matched {
        tracing::info!(
            version = %current,
            "peek_pending_relaunch: post-update launch detected, forcing main window visible"
        );
    }
    matched
}

/// Read + delete the pending_relaunch.json marker. Returns Some(version)
/// only when the marker exists AND the running binary's version matches
/// the marker's target_version — i.e. the install actually upgraded us
/// to what we asked for. Any mismatch silently discards the marker.
fn consume_pending_relaunch() -> Option<String> {
    let path = pending_relaunch_path()?;
    if !path.exists() {
        return None;
    }
    let json = std::fs::read_to_string(&path).ok()?;
    let rec: PendingRelaunch = serde_json::from_str(&json).ok()?;
    let current = env!("CARGO_PKG_VERSION");
    let _ = std::fs::remove_file(&path);
    if rec.target_version == current {
        tracing::info!(version = %current, "consumed pending_relaunch marker — surfacing post-update toast");
        Some(current.to_string())
    } else {
        tracing::info!(
            target = %rec.target_version,
            actual = %current,
            "pending_relaunch marker version mismatch — discarding without surfacing toast"
        );
        None
    }
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

/// Strict three-segment semver comparator: returns true when `candidate`
/// is strictly greater than `baseline`. Anything that fails to parse as
/// `X.Y.Z` (digits only, no pre-release suffix) returns false — better
/// to refuse than to install a malformed version. Matches the server-
/// side parser in `UpdaterApiController::parseSemver` so client + server
/// agree on what counts as "newer".
fn semver_strictly_greater(candidate: &str, baseline: &str) -> bool {
    let c = parse_semver(candidate);
    let b = parse_semver(baseline);
    match (c, b) {
        (Some(c), Some(b)) => c > b,
        _ => false,
    }
}

/// Build the canonical metadata blob whose bytes the operator signed.
/// The format is fixed and MUST match the server-side reconstruction in
/// `App\Services\Releases\UpdateMetadataSigner::canonicalBlob()` and the
/// operator-side signer in `scripts/sign-update-metadata.py`. Any drift
/// in line ordering, separator, or field encoding voids every signature
/// the operator has already issued.
///
/// Format (UTF-8, LF-separated, NO trailing newline):
///
/// ```text
/// utterbar-update-v1
/// version=<X.Y.Z>
/// channel=<stable|beta>
/// size_bytes=<integer>
/// sha256=<lowercase 64-hex>
/// download_url=<https://dl.utterbar.com/...>
/// ```
fn canonical_metadata_blob(
    version: &str,
    channel: &str,
    size_bytes: u64,
    sha256: &str,
    download_url: &str,
) -> Vec<u8> {
    format!(
        "{magic}\nversion={version}\nchannel={channel}\nsize_bytes={size_bytes}\nsha256={sha}\ndownload_url={url}",
        magic = UPDATE_METADATA_MAGIC,
        version = version,
        channel = channel,
        size_bytes = size_bytes,
        sha = sha256.to_ascii_lowercase(),
        url = download_url,
    )
    .into_bytes()
}

/// Verify a base64-encoded Ed25519 signature against the canonical metadata
/// blob built from the supplied fields. Returns false on any failure mode
/// (bad base64, wrong length, unparseable key, signature mismatch) — the
/// caller treats false as "do not install" and bails loudly.
fn verify_update_signature(
    signature_b64: &str,
    version: &str,
    channel: &str,
    size_bytes: u64,
    sha256: &str,
    download_url: &str,
) -> bool {
    let sig_bytes = match base64::engine::general_purpose::STANDARD.decode(signature_b64) {
        Ok(b) => b,
        Err(_) => return false,
    };
    if sig_bytes.len() != 64 {
        return false;
    }
    let sig_arr: [u8; 64] = match sig_bytes.try_into() {
        Ok(a) => a,
        Err(_) => return false,
    };
    let signature = Signature::from_bytes(&sig_arr);

    let vk = match VerifyingKey::from_bytes(&UPDATER_PUBLIC_KEY) {
        Ok(k) => k,
        Err(_) => return false,
    };

    let blob = canonical_metadata_blob(version, channel, size_bytes, sha256, download_url);
    vk.verify_strict(&blob, &signature).is_ok()
}

fn parse_semver(s: &str) -> Option<(u32, u32, u32)> {
    let mut it = s.split('.');
    let major = it.next()?.parse().ok()?;
    let minor = it.next()?.parse().ok()?;
    let patch = it.next()?.parse().ok()?;
    if it.next().is_some() {
        return None; // reject anything beyond X.Y.Z
    }
    Some((major, minor, patch))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semver_strictly_greater_basic() {
        assert!(semver_strictly_greater("0.8.8", "0.8.7"));
        assert!(semver_strictly_greater("0.9.0", "0.8.99"));
        assert!(semver_strictly_greater("1.0.0", "0.99.99"));
        assert!(!semver_strictly_greater("0.8.7", "0.8.7"));
        assert!(!semver_strictly_greater("0.8.6", "0.8.7"));
    }

    #[test]
    fn semver_strictly_greater_rejects_garbage() {
        assert!(!semver_strictly_greater("foo", "0.8.7"));
        assert!(!semver_strictly_greater("0.8", "0.8.7"));
        assert!(!semver_strictly_greater("0.8.7-beta", "0.8.7"));
        assert!(!semver_strictly_greater("0.8.7.1", "0.8.7"));
    }

    #[test]
    fn canonical_blob_matches_spec() {
        // The exact byte layout the operator-side signer + server-side
        // verifier both reconstruct. If this test ever needs editing, the
        // format has drifted and every previously-signed release is
        // invalidated — bump UPDATE_METADATA_MAGIC to v2 first.
        let blob = canonical_metadata_blob(
            "0.9.0",
            "stable",
            12345678,
            "ABCDEF0123456789aBcDeF0123456789ABCDEF0123456789ABCDEF0123456789",
            "https://dl.utterbar.com/releases/0.9.0/Utterbar-Setup-0.9.0.exe",
        );
        let expected = b"utterbar-update-v1\nversion=0.9.0\nchannel=stable\nsize_bytes=12345678\nsha256=abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789\ndownload_url=https://dl.utterbar.com/releases/0.9.0/Utterbar-Setup-0.9.0.exe";
        assert_eq!(blob.as_slice(), expected.as_slice());
    }

    #[test]
    fn verify_rejects_obviously_bad_signature() {
        // base64-decoded but wrong length
        assert!(!verify_update_signature(
            "AAAA",
            "0.9.0",
            "stable",
            1,
            "00",
            "https://dl.utterbar.com/x"
        ));
        // not base64 at all
        assert!(!verify_update_signature(
            "not-base-64-!!!",
            "0.9.0",
            "stable",
            1,
            "00",
            "https://dl.utterbar.com/x"
        ));
        // valid 64-byte zero signature against zero pubkey: ed25519
        // identity-key checks SHOULD reject — `verify_strict` treats
        // small-order keys as unsafe.
        let zeroes = base64::engine::general_purpose::STANDARD.encode([0u8; 64]);
        assert!(!verify_update_signature(
            &zeroes,
            "0.9.0",
            "stable",
            1,
            "00",
            "https://dl.utterbar.com/x"
        ));
    }

    /// Pins the on-disk JSON shape of `pending_relaunch.json` so a
    /// future struct rename can't silently break the post-update
    /// toast AND the post-update force-show window decision. Both
    /// consumers parse this exact shape; if the field name drifts,
    /// every freshly-installed user lands silently in the tray
    /// after an update and the "Updated to vX.Y.Z" toast disappears.
    #[test]
    fn pending_relaunch_json_shape_is_stable() {
        let rec = PendingRelaunch {
            target_version: "0.16.2".to_string(),
            written_at_unix: 1_700_000_000,
        };
        let json = serde_json::to_string(&rec).expect("serialise");
        // Field names are load-bearing — both peek and consume read
        // them as `target_version` / `written_at_unix`. Don't rename
        // without bumping the marker file format.
        assert!(
            json.contains("\"target_version\":\"0.16.2\""),
            "expected target_version field, got: {json}"
        );
        assert!(
            json.contains("\"written_at_unix\":1700000000"),
            "expected written_at_unix field, got: {json}"
        );

        // Roundtrip — the consumer-side `serde_json::from_str` must
        // accept what the producer wrote.
        let parsed: PendingRelaunch = serde_json::from_str(&json).expect("deserialise");
        assert_eq!(parsed.target_version, "0.16.2");
        assert_eq!(parsed.written_at_unix, 1_700_000_000);

        // The historic on-disk shape (literally written by 0.16.x
        // installs) MUST also still parse — this is the failure mode
        // the test exists to catch. If a future struct change makes
        // either of these strings fail to deserialise, the next
        // post-update launch from the previous version cannot
        // surface its toast or force-show its window.
        let historic_one_liner = r#"{"target_version":"0.16.2","written_at_unix":1700000000}"#;
        let historic: PendingRelaunch = serde_json::from_str(historic_one_liner).expect(
            "historic 0.16.x marker shape must keep parsing — see test for rename guidance",
        );
        assert_eq!(historic.target_version, "0.16.2");
    }
}
