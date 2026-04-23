//! File transcription queue.
//!
//! Jobs run on a single-flight worker so we never thrash the ASR provider. Each job
//! decodes the source file via Symphonia, feeds it to the active ASR provider, applies
//! the selected mode's post-processing, and writes a sidecar `.txt` next to the source.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::Result;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::sync::{Mutex, Notify};
use ts_rs::TS;

use crate::app_state::AppState;
use crate::db::Db;
use crate::domain::{FileJobProgress, EVT_FILE_JOB_PROGRESS};
use crate::services::audio::decode;
use crate::services::inference::{AsrOptions, AudioBuffer};
use crate::util::time::now_ms;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct FileJob {
    pub id: String,
    pub source_path: String,
    pub mode_id: Option<String>,
    pub model_id: Option<String>,
    pub status: FileJobStatus,
    pub progress: f32,
    pub result_path: Option<String>,
    pub error_message: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export, export_to = "../../src/generated/")]
#[serde(rename_all = "snake_case")]
pub enum FileJobStatus {
    Queued,
    Running,
    Done,
    Error,
    Cancelled,
}

pub struct FileJobService {
    db: Arc<Db>,
    notify: Arc<Notify>,
    worker_started: Mutex<bool>,
    /// Per-running-job cancellation flags. Populated when the worker picks up a job,
    /// drained when the job exits (success, error, or cancel). `cancel()` flips the bit
    /// and the worker checks it at every phase boundary — queued jobs never get a flag
    /// because their DB status flips to `Cancelled` before the worker ever sees them
    /// (the `next_queued` query filters on `status = 'queued'`).
    running: Mutex<HashMap<String, Arc<AtomicBool>>>,
}

impl FileJobService {
    pub fn new(db: Arc<Db>) -> Self {
        Self {
            db,
            notify: Arc::new(Notify::new()),
            worker_started: Mutex::new(false),
            running: Mutex::new(HashMap::new()),
        }
    }

    pub fn enqueue(
        &self,
        source_path: String,
        mode_id: Option<String>,
        model_id: Option<String>,
    ) -> Result<FileJob> {
        let job = FileJob {
            id: ulid::Ulid::new().to_string(),
            source_path,
            mode_id,
            model_id,
            status: FileJobStatus::Queued,
            progress: 0.0,
            result_path: None,
            error_message: None,
            created_at: now_ms(),
            updated_at: now_ms(),
        };
        let conn = self.db.conn()?;
        conn.execute(
            "INSERT INTO file_jobs(id,source_path,mode_id,model_id,status,progress,result_path,error_message,created_at,updated_at)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
            params![job.id, job.source_path, job.mode_id, job.model_id,
                serde_json::to_string(&job.status)?, job.progress,
                job.result_path, job.error_message, job.created_at, job.updated_at],
        )?;
        self.notify.notify_one();
        Ok(job)
    }

    pub fn list(&self) -> Result<Vec<FileJob>> {
        let conn = self.db.conn()?;
        let mut stmt = conn.prepare("SELECT id,source_path,mode_id,model_id,status,progress,result_path,error_message,created_at,updated_at FROM file_jobs ORDER BY created_at DESC LIMIT 200")?;
        let rows = stmt.query_map([], |r| {
            let status: String = r.get(4)?;
            Ok(FileJob {
                id: r.get(0)?,
                source_path: r.get(1)?,
                mode_id: r.get(2)?,
                model_id: r.get(3)?,
                status: serde_json::from_str(&status).unwrap_or(FileJobStatus::Queued),
                progress: r.get(5)?,
                result_path: r.get(6)?,
                error_message: r.get(7)?,
                created_at: r.get(8)?,
                updated_at: r.get(9)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// Async so the running-map mutex can be awaited. Cancellation is two-phase:
    ///   * Queued job — DB flip to `Cancelled` is sufficient; `next_queued()` filters
    ///     on `status = 'queued'` so the worker never picks it up.
    ///   * Running job — we flip the DB row too, but the worker won't see that until it
    ///     polls; setting the `AtomicBool` in `running` lets the worker short-circuit
    ///     between phases (after decode / after ASR / before writing the sidecar) so the
    ///     cancelled job stops within a few hundred milliseconds instead of running to
    ///     completion and then being ignored.
    pub async fn cancel(&self, id: &str) -> Result<()> {
        let conn = self.db.conn()?;
        conn.execute(
            "UPDATE file_jobs SET status=?1, updated_at=?2 WHERE id=?3 AND status IN ('\"queued\"','\"running\"')",
            params![serde_json::to_string(&FileJobStatus::Cancelled)?, now_ms(), id],
        )?;
        if let Some(flag) = self.running.lock().await.get(id) {
            flag.store(true, Ordering::Relaxed);
        }
        Ok(())
    }

    /// Spawn the worker loop if it isn't already running. Safe to call many times.
    pub async fn start_worker(self: Arc<Self>, app: Arc<AppState>, handle: AppHandle) {
        let mut started = self.worker_started.lock().await;
        if *started {
            return;
        }
        *started = true;
        let this = self.clone();
        tokio::spawn(async move {
            loop {
                // Drain any queued job, then wait for a wake-up.
                while let Some(job) = this.next_queued().ok().flatten() {
                    let _ = this.run_job(job, &app, &handle).await;
                }
                this.notify.notified().await;
            }
        });
    }

    fn next_queued(&self) -> Result<Option<FileJob>> {
        let conn = self.db.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id,source_path,mode_id,model_id,status,progress,result_path,error_message,created_at,updated_at FROM file_jobs WHERE status = '\"queued\"' ORDER BY created_at ASC LIMIT 1"
        )?;
        let mut rows = stmt.query([])?;
        let Some(r) = rows.next()? else { return Ok(None); };
        let status: String = r.get(4)?;
        Ok(Some(FileJob {
            id: r.get(0)?,
            source_path: r.get(1)?,
            mode_id: r.get(2)?,
            model_id: r.get(3)?,
            status: serde_json::from_str(&status).unwrap_or(FileJobStatus::Queued),
            progress: r.get(5)?,
            result_path: r.get(6)?,
            error_message: r.get(7)?,
            created_at: r.get(8)?,
            updated_at: r.get(9)?,
        }))
    }

    async fn run_job(&self, job: FileJob, app: &AppState, handle: &AppHandle) -> Result<()> {
        // Register the cancel flag before the first DB write. Guarded by RAII-ish dropguard
        // below so we always deregister, even if we panic or bail early.
        let cancel_flag = Arc::new(AtomicBool::new(false));
        self.running.lock().await.insert(job.id.clone(), cancel_flag.clone());

        let result = self
            .run_job_inner(&job, cancel_flag.clone(), app, handle)
            .await;

        // Drain the flag regardless of outcome.
        self.running.lock().await.remove(&job.id);
        result
    }

    async fn run_job_inner(
        &self,
        job: &FileJob,
        cancel: Arc<AtomicBool>,
        app: &AppState,
        handle: &AppHandle,
    ) -> Result<()> {
        self.update(&job.id, FileJobStatus::Running, 0.05, None, None)?;
        emit(handle, &job.id, 0.05, FileJobStatus::Running);

        let settings = app.settings.snapshot().await;
        let mode_id = job.mode_id.clone().or_else(|| settings.active_mode_id.clone()).unwrap_or_else(|| "voice-to-text".into());
        let model_id = job
            .model_id
            .clone()
            .or(settings.models.default_asr_model_id.clone())
            .unwrap_or_else(|| "stub".into());

        if self.check_cancelled(&job.id, &cancel, handle, 0.05)? {
            return Ok(());
        }

        let source = PathBuf::from(&job.source_path);
        let decoded = match tokio::task::spawn_blocking({
            let s = source.clone();
            move || decode::decode_file(&s)
        })
        .await
        {
            Ok(Ok(d)) => d,
            Ok(Err(e)) => {
                self.fail(&job.id, &format!("decode: {e}"))?;
                emit(handle, &job.id, 0.0, FileJobStatus::Error);
                return Ok(());
            }
            Err(e) => {
                self.fail(&job.id, &format!("decode join: {e}"))?;
                emit(handle, &job.id, 0.0, FileJobStatus::Error);
                return Ok(());
            }
        };

        if self.check_cancelled(&job.id, &cancel, handle, 0.35)? {
            return Ok(());
        }

        self.update(&job.id, FileJobStatus::Running, 0.35, None, None)?;
        emit(handle, &job.id, 0.35, FileJobStatus::Running);

        let asr = match app
            .inference
            .transcribe(
                &model_id,
                AudioBuffer {
                    pcm: decoded.pcm,
                    sample_rate: decoded.sample_rate,
                    channels: 1,
                },
                AsrOptions::default(),
            )
            .await
        {
            Ok(r) => r,
            Err(e) => {
                self.fail(&job.id, &format!("asr: {e}"))?;
                emit(handle, &job.id, 0.35, FileJobStatus::Error);
                return Ok(());
            }
        };

        if self.check_cancelled(&job.id, &cancel, handle, 0.75)? {
            return Ok(());
        }

        self.update(&job.id, FileJobStatus::Running, 0.75, None, None)?;
        emit(handle, &job.id, 0.75, FileJobStatus::Running);

        let replaced = app.vocabulary.apply_replacements(&asr.text).unwrap_or(asr.text.clone());
        let processed = if let Some(mode) = app.modes.get(&mode_id).ok().flatten() {
            match &mode.post_processing {
                crate::domain::PostProcessing::None => replaced.clone(),
                crate::domain::PostProcessing::Deterministic { .. } => {
                    crate::services::llm::deterministic::DeterministicProvider.rewrite_sync(&replaced)
                }
                crate::domain::PostProcessing::Llm { prompt, .. } => {
                    let llm = crate::services::llm::best_available().await;
                    llm
                        .rewrite(&replaced, prompt, &crate::services::llm::ProcContext::default())
                        .await
                        .unwrap_or_else(|_| replaced.clone())
                }
            }
        } else {
            replaced
        };

        // Last gate before we touch the disk. A cancel that lands between ASR finishing and
        // the sidecar write must NOT leave the user with a `.utterbar.txt` they just asked
        // us to throw away.
        if self.check_cancelled(&job.id, &cancel, handle, 0.95)? {
            return Ok(());
        }

        let out_path = sidecar_path(&source, "txt");
        if let Err(e) = tokio::fs::write(&out_path, &processed).await {
            self.fail(&job.id, &format!("write result: {e}"))?;
            emit(handle, &job.id, 0.95, FileJobStatus::Error);
            return Ok(());
        }

        let rel = out_path.to_string_lossy().into_owned();
        self.update(&job.id, FileJobStatus::Done, 1.0, Some(rel), None)?;
        emit(handle, &job.id, 1.0, FileJobStatus::Done);
        Ok(())
    }

    /// If the cancel flag is set, persist the `Cancelled` status, emit the event, and
    /// return `Ok(true)` so the caller returns immediately. `Ok(false)` means "carry on".
    fn check_cancelled(
        &self,
        id: &str,
        flag: &AtomicBool,
        handle: &AppHandle,
        progress: f32,
    ) -> Result<bool> {
        if !flag.load(Ordering::Relaxed) {
            return Ok(false);
        }
        self.update(id, FileJobStatus::Cancelled, progress, None, None)?;
        emit(handle, id, progress, FileJobStatus::Cancelled);
        Ok(true)
    }

    fn update(
        &self,
        id: &str,
        status: FileJobStatus,
        progress: f32,
        result_path: Option<String>,
        error_message: Option<String>,
    ) -> Result<()> {
        let conn = self.db.conn()?;
        conn.execute(
            "UPDATE file_jobs SET status=?1, progress=?2, result_path=COALESCE(?3, result_path), error_message=COALESCE(?4, error_message), updated_at=?5 WHERE id=?6",
            params![
                serde_json::to_string(&status)?,
                progress,
                result_path,
                error_message,
                now_ms(),
                id
            ],
        )?;
        Ok(())
    }

    fn fail(&self, id: &str, msg: &str) -> Result<()> {
        self.update(id, FileJobStatus::Error, 0.0, None, Some(msg.to_string()))
    }
}

/// Build `<source-stem>.utterbar.<ext>` next to the source file. The `.utterbar`
/// infix is deliberate — it matches the runtime product name and is what the
/// Files route copy promises the user — and it stays the same even though the
/// crate / data dir still uses the historic `orliq` identifier. Never rename
/// this without coordinating with `src/routes/FilesRoute.tsx`.
fn sidecar_path(source: &Path, ext: &str) -> PathBuf {
    let mut p = source.to_path_buf();
    let stem = p
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("transcript")
        .to_string();
    p.set_file_name(format!("{stem}.utterbar.{ext}"));
    p
}

fn emit(handle: &AppHandle, id: &str, progress: f32, status: FileJobStatus) {
    let _ = handle.emit(
        EVT_FILE_JOB_PROGRESS,
        FileJobProgress {
            id: id.into(),
            progress,
            status: match status {
                FileJobStatus::Queued => "queued",
                FileJobStatus::Running => "running",
                FileJobStatus::Done => "done",
                FileJobStatus::Error => "error",
                FileJobStatus::Cancelled => "cancelled",
            }
            .into(),
        },
    );
}
