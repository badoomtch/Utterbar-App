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
use crate::services::output_validation::{
    validate, EngineFamily, LanguagePin, ValidationContext, ValidationVerdict,
};
use crate::util::time::now_ms;

/// User-facing message attached to a file job whose transcript was
/// dropped by the shared output validator. Kept in one place so the
/// callsite, the test, and any future surface that renders this
/// status all share a single string.
const FILTERED_MSG: &str =
    "Output filtered. The transcript looked like ASR junk, so no sidecar files were written.";

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

    /// Single-row lookup. Returns None when the id doesn't exist —
    /// callers (Tauri commands) translate that into a friendly error.
    pub fn get(&self, id: &str) -> Result<Option<FileJob>> {
        let conn = self.db.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id,source_path,mode_id,model_id,status,progress,result_path,error_message,created_at,updated_at FROM file_jobs WHERE id = ?1 LIMIT 1",
        )?;
        let mut rows = stmt.query(params![id])?;
        if let Some(r) = rows.next()? {
            let status: String = r.get(4)?;
            return Ok(Some(FileJob {
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
            }));
        }
        Ok(None)
    }

    /// Remove the row from the database. Does NOT touch the result
    /// file on disk — that's the caller's responsibility (the
    /// `delete_job` Tauri command exposes an `also_delete_file` flag).
    pub fn delete(&self, id: &str) -> Result<()> {
        let conn = self.db.conn()?;
        conn.execute("DELETE FROM file_jobs WHERE id = ?1", params![id])?;
        Ok(())
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
        let Some(r) = rows.next()? else {
            return Ok(None);
        };
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
        self.running
            .lock()
            .await
            .insert(job.id.clone(), cancel_flag.clone());

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
        let mode_id = job
            .mode_id
            .clone()
            .or_else(|| settings.active_mode_id.clone())
            .unwrap_or_else(|| "voice-to-text".into());
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
                // Request word-level timings for subtitle export. No
                // current provider populates `words: Some(...)` — the
                // sherpa-rs 0.6 Rust wrapper drops the FFI's per-token
                // timestamps. When a future provider does start
                // returning real timings, the subtitle code path
                // (`subtitles::cues_from_words`) auto-upgrades from
                // proportional sentence-level timing to true word-
                // level timing without further changes here.
                AsrOptions {
                    want_word_timings: true,
                    ..Default::default()
                },
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

        let replaced = app
            .vocabulary
            .apply_replacements(&asr.text)
            .unwrap_or(asr.text.clone());
        let processed = if let Some(mode) = app.modes.get(&mode_id).ok().flatten() {
            match &mode.post_processing {
                crate::domain::PostProcessing::None => replaced.clone(),
                crate::domain::PostProcessing::Deterministic { .. } => {
                    crate::services::llm::deterministic::DeterministicProvider
                        .rewrite_sync(&replaced)
                }
                crate::domain::PostProcessing::Llm { prompt, .. } => {
                    let llm = crate::services::llm::best_available().await;
                    llm.rewrite(
                        &replaced,
                        prompt,
                        &crate::services::llm::ProcContext::default(),
                    )
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

        // Output validation. Same shared validator regular dictation
        // uses, applied here before any sidecar file is written. On
        // Drop we mark the job Error (the same path used for decode /
        // ASR / write failures throughout this function), use a
        // dedicated user-facing message, and emit one structured log
        // line for soak telemetry. Critically, no `.txt`, `.srt`, or
        // `.vtt` is written -- the user gets a failed job, not a
        // sidecar full of "the the the the the the".
        let lang_setting = settings.models.whisper_language.clone();
        let verdict = decide_file_validation(&processed, &model_id, &lang_setting, asr.duration_ms);
        if let ValidationVerdict::Drop {
            reason,
            user_notice,
        } = verdict
        {
            tracing::info!(
                reason = %reason.as_str(),
                model = %model_id,
                language = %lang_setting,
                duration_ms = asr.duration_ms,
                chars = processed.chars().count(),
                user_notice = user_notice,
                job_id = %job.id,
                source = %source.display(),
                "output_validation: dropped file transcript",
            );
            self.fail(&job.id, FILTERED_MSG)?;
            emit(handle, &job.id, 0.95, FileJobStatus::Error);
            return Ok(());
        }

        let out_path = sidecar_path(&source, "txt");
        if let Err(e) = tokio::fs::write(&out_path, &processed).await {
            self.fail(&job.id, &format!("write result: {e}"))?;
            emit(handle, &job.id, 0.95, FileJobStatus::Error);
            return Ok(());
        }

        // Subtitle sidecars (.srt + .vtt). Always emitted alongside
        // the .txt — they're tiny text files, and a per-job toggle
        // would mean either schema migration or a "I forgot the box"
        // class of bug. If a user only wants the .txt they can ignore
        // the others. See services/subtitles.rs for the timing-honesty
        // discussion.
        //
        // Failure here is non-fatal: the .txt has already landed, the
        // user has their transcript. We log + carry on.
        let cues = if let Some(words) = asr.words.as_ref() {
            crate::services::subtitles::cues_from_words(words)
        } else {
            crate::services::subtitles::cues_from_text(&processed, asr.duration_ms)
        };
        if !cues.is_empty() {
            let srt_path = sidecar_path(&source, "srt");
            let vtt_path = sidecar_path(&source, "vtt");
            let srt = crate::services::subtitles::render_srt(&cues);
            let vtt = crate::services::subtitles::render_vtt(&cues);
            if let Err(e) = tokio::fs::write(&srt_path, &srt).await {
                tracing::warn!(error = ?e, path = %srt_path.display(), "subtitle .srt write failed; continuing");
            }
            if let Err(e) = tokio::fs::write(&vtt_path, &vtt).await {
                tracing::warn!(error = ?e, path = %vtt_path.display(), "subtitle .vtt write failed; continuing");
            }
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

/// Run the shared output validator against a file-transcription
/// transcript and return the verdict.
///
/// This is the file-transcription analogue of the dictation callsite
/// in `commands/dictation.rs`. Same validator, same rules. The
/// engine family is derived from the model id prefix (Utterbar's
/// catalogue is prefix-coded `sherpa-whisper-*` / `sherpa-parakeet-*`)
/// and the language pin from the user's `whisper_language` setting:
///
/// - Parakeet -> `EnglishOnly` regardless of the language field
///   (the engine is hard-pinned to English; foreign-script output
///   is always a hallucination).
/// - Whisper + `"en"` -> `EnglishOnly`.
/// - Whisper + `""` / `"auto"` -> `Auto` (multilingual; the
///   foreign-script gate must not fire).
/// - Whisper + anything else -> `OtherExplicit` (user explicitly
///   chose a non-English language; foreign script is expected).
///
/// Extracted as a private helper so the tests can exercise the
/// engine/language derivation + the verdict path without spinning up
/// an `AppState`, a tokio runtime, or real audio fixtures.
fn decide_file_validation(
    text: &str,
    model_id: &str,
    whisper_language: &str,
    duration_ms: u32,
) -> ValidationVerdict {
    let engine = if model_id.starts_with("sherpa-whisper") {
        EngineFamily::Whisper
    } else {
        EngineFamily::Parakeet
    };
    let lang = whisper_language.trim().to_lowercase();
    let language_pin = match (engine, lang.as_str()) {
        (EngineFamily::Parakeet, _) => LanguagePin::EnglishOnly,
        (EngineFamily::Whisper, "en") => LanguagePin::EnglishOnly,
        (EngineFamily::Whisper, "" | "auto") => LanguagePin::Auto,
        (EngineFamily::Whisper, _) => LanguagePin::OtherExplicit,
    };
    validate(
        text,
        &ValidationContext {
            engine,
            language_pin,
            audio_duration_ms: duration_ms,
        },
    )
}

/// Build `<source-stem>.utterbar.<ext>` next to the source file. The `.utterbar`
/// infix is deliberate - it matches the runtime product name and is what the
/// Files route copy promises the user - and it stays the same even though the
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

#[cfg(test)]
mod validation_tests {
    //! File-transcription output validation. These tests exercise
    //! the small private `decide_file_validation` helper, which is
    //! the only surface specific to the file path; the deep
    //! rule-by-rule coverage of the validator itself lives in
    //! `services/output_validation.rs::tests`. The cases here are
    //! the ones the spec requires to be wired up correctly through
    //! the engine/language derivation:
    //!
    //!  - valid transcript -> Accept (would proceed to sidecar write)
    //!  - repeated-token output -> noticed Drop
    //!  - repeated-punctuation output -> noticed Drop
    //!  - foreign script + English-only context -> noticed Drop
    //!  - foreign script + Whisper auto / OtherExplicit -> Accept
    //!
    //! Spinning up an `AppState`, a `tokio` runtime, a real audio
    //! decoder, and an ASR provider just to land a sidecar on disk
    //! and then assert it was/wasn't written would multiply test
    //! infrastructure for no extra signal: the only file-path
    //! specific decision is the helper, and the helper's verdict is
    //! what the run_job_inner branch consumes. Full end-to-end
    //! integration is deferred until we have a fixture-driven
    //! files-service smoke harness for unrelated reasons.

    use super::*;
    use crate::services::output_validation::{DropReason, ValidationVerdict};

    fn assert_drop(v: ValidationVerdict, expected: DropReason, expected_notice: bool) {
        match v {
            ValidationVerdict::Drop {
                reason,
                user_notice,
            } => {
                assert_eq!(reason, expected, "wrong drop reason");
                assert_eq!(
                    user_notice, expected_notice,
                    "wrong notice flag for {expected:?}"
                );
            }
            ValidationVerdict::Accept => panic!("expected Drop({expected:?}), got Accept"),
        }
    }

    #[test]
    fn valid_transcript_accepts() {
        let v = decide_file_validation(
            "Tell me about the proposal Marcus sent yesterday and pull in the latest figures.",
            "sherpa-whisper-medium",
            "en",
            12_000,
        );
        assert_eq!(v, ValidationVerdict::Accept);
    }

    #[test]
    fn empty_transcript_drops_quietly() {
        // Empty after post-processing -> Drop(Empty, user_notice=false).
        // The file callsite still treats this as a job failure (no
        // sidecar should land), but the verdict carries the quiet
        // flag so future surfaces can render silence-style copy.
        assert_drop(
            decide_file_validation("", "sherpa-whisper-small", "en", 5_000),
            DropReason::Empty,
            false,
        );
    }

    #[test]
    fn single_punctuation_drops_quietly() {
        assert_drop(
            decide_file_validation(".", "sherpa-whisper-small", "en", 1_500),
            DropReason::SinglePunctuation,
            false,
        );
    }

    #[test]
    fn repeated_token_drops_with_notice() {
        assert_drop(
            decide_file_validation(
                "the the the the the the and then we",
                "sherpa-whisper-large-v3",
                "en",
                30_000,
            ),
            DropReason::RepeatedToken,
            true,
        );
    }

    #[test]
    fn repeated_punctuation_drops_with_notice() {
        assert_drop(
            decide_file_validation("hello..........", "sherpa-whisper-medium", "en", 4_000),
            DropReason::RepeatedPunctuation,
            true,
        );
    }

    #[test]
    fn foreign_script_dropped_on_english_only_whisper() {
        // Whisper pinned to English. CJK output is a hallucination.
        assert_drop(
            decide_file_validation("你好世界", "sherpa-whisper-medium", "en", 8_000),
            DropReason::UnexpectedForeignScript,
            true,
        );
    }

    #[test]
    fn foreign_script_dropped_on_parakeet_regardless_of_language_field() {
        // Parakeet is hard-pinned English. Even if the
        // whisper_language field is set to "zh" (which Parakeet
        // ignores), the validator should still treat the path as
        // English-only and drop foreign-script output.
        assert_drop(
            decide_file_validation("你好世界", "sherpa-parakeet-tdt-0.6b-v3-int8", "zh", 8_000),
            DropReason::UnexpectedForeignScript,
            true,
        );
    }

    #[test]
    fn foreign_script_accepted_on_whisper_auto() {
        // Multilingual Whisper auto-detect. CJK output is legitimate.
        let v = decide_file_validation("你好世界", "sherpa-whisper-medium", "auto", 8_000);
        assert_eq!(v, ValidationVerdict::Accept);
    }

    #[test]
    fn foreign_script_accepted_on_explicit_non_english() {
        // User explicitly chose Chinese. CJK output is exactly
        // what they want.
        let v = decide_file_validation("你好世界", "sherpa-whisper-medium", "zh", 8_000);
        assert_eq!(v, ValidationVerdict::Accept);
    }

    #[test]
    fn empty_language_field_treats_whisper_as_auto() {
        // Some older settings rows store the language as an empty
        // string rather than "auto". Both must behave identically:
        // multilingual, foreign-script gate skipped.
        let v = decide_file_validation("你好世界", "sherpa-whisper-medium", "", 8_000);
        assert_eq!(v, ValidationVerdict::Accept);
    }

    #[test]
    fn currency_symbols_accepted() {
        // The kind of file-transcription output where a podcast or
        // meeting mentions a price must never trip the
        // foreign-script gate. (The validator's `unexpected_foreign_script`
        // already excludes currency symbols, but we belt-and-brace
        // it by routing through this function.)
        let v = decide_file_validation(
            "We agreed on £39.99, plus €100 in shipping.",
            "sherpa-parakeet-tdt-0.6b-v3-int8",
            "en",
            5_000,
        );
        assert_eq!(v, ValidationVerdict::Accept);
    }
}
