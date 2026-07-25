use std::sync::Arc;

use tauri::State;

use crate::app_state::AppState;
use crate::services::files::FileJob;
use crate::util::result::{CmdResult, OrliqError};

#[tauri::command]
pub async fn enqueue_file(
    state: State<'_, Arc<AppState>>,
    source_path: String,
    mode_id: Option<String>,
    model_id: Option<String>,
) -> CmdResult<FileJob> {
    state
        .files
        .enqueue(source_path, mode_id, model_id)
        .map_err(OrliqError::from)
}

/// Frontend-friendly entry point: takes any path and routes to the
/// right enqueue path. Single round trip — the frontend doesn't need to
/// `stat()` first to decide between file and folder. Returns the number
/// of jobs enqueued (1 for a single file, 0..=1000 for a folder).
#[tauri::command]
pub async fn enqueue_path(
    state: State<'_, Arc<AppState>>,
    path: String,
    mode_id: Option<String>,
    model_id: Option<String>,
) -> CmdResult<usize> {
    let p = std::path::Path::new(&path);
    if p.is_dir() {
        return enqueue_folder(state, path, mode_id, model_id).await;
    }
    enqueue_file(state, path, mode_id, model_id)
        .await
        .map(|_| 1)
}

/// Walk a folder recursively and enqueue every supported audio/video file
/// found. Returns the number of jobs created. Bounded at 1,000 enqueued
/// files per call so a user who drops their music library doesn't fill
/// the queue with thousands of mp3s before they realise — they get the
/// first 1k and a clear count back. Anything past that they can re-drop
/// after the queue drains.
///
/// Extension filter mirrors the dropzone copy on the Files page —
/// keeping these in sync is on the maintainer; if you add formats here,
/// update both `pickFiles` filters and the dropzone hint text.
#[tauri::command]
pub async fn enqueue_folder(
    state: State<'_, Arc<AppState>>,
    folder_path: String,
    mode_id: Option<String>,
    model_id: Option<String>,
) -> CmdResult<usize> {
    const ALLOWED: &[&str] = &["mp3", "wav", "m4a", "flac", "ogg", "mp4", "mov", "mkv"];
    const HARD_CAP: usize = 1_000;

    let root = std::path::PathBuf::from(&folder_path);
    if !root.is_dir() {
        return Err(OrliqError::Internal {
            message: format!("Not a folder: {}", folder_path),
        });
    }

    let mut paths: Vec<String> = Vec::new();
    walk_for_audio(&root, ALLOWED, &mut paths, HARD_CAP);

    if paths.is_empty() {
        return Ok(0);
    }

    let mut count = 0usize;
    for p in paths {
        match state.files.enqueue(p, mode_id.clone(), model_id.clone()) {
            Ok(_) => count += 1,
            Err(e) => {
                tracing::warn!(?e, "enqueue_folder: per-file enqueue failed; continuing");
            }
        }
    }
    Ok(count)
}

/// Recursive directory walk. Plain `std::fs` so we don't pull a new
/// crate in. Bounded by HARD_CAP — once we hit it, return immediately
/// so a giant tree doesn't keep spinning. Symlinks are NOT followed —
/// avoids accidental cycles and out-of-tree captures.
fn walk_for_audio(dir: &std::path::Path, allowed: &[&str], out: &mut Vec<String>, hard_cap: usize) {
    if out.len() >= hard_cap {
        return;
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        if out.len() >= hard_cap {
            return;
        }
        let path = entry.path();
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_dir() {
            walk_for_audio(&path, allowed, out, hard_cap);
            continue;
        }
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            let lower = ext.to_ascii_lowercase();
            if allowed.iter().any(|a| *a == lower) {
                if let Some(s) = path.to_str() {
                    out.push(s.to_string());
                }
            }
        }
    }
}

#[tauri::command]
pub async fn list_jobs(state: State<'_, Arc<AppState>>) -> CmdResult<Vec<FileJob>> {
    state.files.list().map_err(OrliqError::from)
}

#[tauri::command]
pub async fn cancel_job(state: State<'_, Arc<AppState>>, id: String) -> CmdResult<()> {
    state.files.cancel(&id).await.map_err(OrliqError::from)
}

/// Read the transcript file produced by a completed job. Returns the
/// full text in-memory — these files are bounded by the source audio
/// length, so even a 90-minute meeting is comfortably under a few MB
/// of text. The IPC bridge handles the string round-trip without
/// chunking. Errors when the job isn't done, the result_path is
/// missing, or the file has been deleted/moved off-disk since.
#[tauri::command]
pub async fn read_job_transcript(state: State<'_, Arc<AppState>>, id: String) -> CmdResult<String> {
    let job = state
        .files
        .get(&id)
        .map_err(OrliqError::from)?
        .ok_or_else(|| OrliqError::Internal {
            message: "Job not found.".into(),
        })?;
    let path = job.result_path.ok_or_else(|| OrliqError::Internal {
        message: "Transcript not available — the job may still be running or have failed.".into(),
    })?;
    tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| OrliqError::Internal {
            message: format!("Could not read transcript: {e}"),
        })
}

/// Open Windows Explorer with the result file selected. Falls back to
/// opening the containing folder if the file has been moved/deleted.
/// Uses `explorer /select,<path>` which is the standard idiom for
/// "show in folder" on Windows.
#[tauri::command]
pub async fn reveal_job_in_folder(state: State<'_, Arc<AppState>>, id: String) -> CmdResult<()> {
    let job = state
        .files
        .get(&id)
        .map_err(OrliqError::from)?
        .ok_or_else(|| OrliqError::Internal {
            message: "Job not found.".into(),
        })?;
    let path = job.result_path.ok_or_else(|| OrliqError::Internal {
        message: "No result file to reveal.".into(),
    })?;

    #[cfg(target_os = "windows")]
    {
        // explorer /select,<path> opens the parent folder and
        // highlights the file. Don't `.spawn()` it through any shell
        // wrapper — the `/select,` syntax has to be a single argument.
        let _ = std::process::Command::new("explorer")
            .arg(format!("/select,{}", path))
            .spawn();
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = path;
        Err(OrliqError::Internal {
            message: "Reveal-in-folder is Windows-only.".into(),
        })
    }
}

/// Delete a job from the list. Pass `also_delete_file = true` to also
/// remove the .utterbar.txt sidecar from disk; defaults to false so the
/// user's transcript files are safe by default. The source audio file
/// is NEVER touched.
#[tauri::command]
pub async fn delete_job(
    state: State<'_, Arc<AppState>>,
    id: String,
    also_delete_file: Option<bool>,
) -> CmdResult<()> {
    let job = state.files.get(&id).map_err(OrliqError::from)?;
    if let (Some(j), Some(true)) = (job.as_ref(), also_delete_file) {
        if let Some(p) = j.result_path.as_ref() {
            let _ = tokio::fs::remove_file(p).await;
        }
    }
    state.files.delete(&id).map_err(OrliqError::from)
}
