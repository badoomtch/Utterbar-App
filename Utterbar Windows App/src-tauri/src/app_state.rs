//! Central dependency container. Everything touching long-lived state lives on AppState.

use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use anyhow::Result;
use tauri::AppHandle;
use tokio::sync::RwLock;

use crate::db::Db;
use crate::services::analytics::AnalyticsService;
use crate::services::audio::AudioService;
use crate::services::audio_ducking::DuckingService;
use crate::services::context::ContextService;
use crate::services::files::FileJobService;
use crate::services::history::HistoryService;
use crate::services::hotkeys::HotkeyService;
use crate::services::inference::InferenceOrchestrator;
use crate::services::licence::LicenceService;
use crate::services::media::MediaService;
use crate::services::models::ModelRegistry;
use crate::services::modes::ModeService;
use crate::services::settings::SettingsStore;
use crate::services::text_inject::TextInjectService;
use crate::services::vocabulary::VocabularyService;

pub struct AppState {
    /// Owning handle to the sqlite pool. Individual services each hold their own `Arc<Db>`
    /// so the field isn't read directly today, but the central container retains ownership
    /// so pool lifetime is tied to the app, not to whichever service happens to outlive the others.
    #[allow(dead_code)]
    pub db: Arc<Db>,
    pub settings: Arc<SettingsStore>,
    pub modes: Arc<ModeService>,
    pub models: Arc<ModelRegistry>,
    pub vocabulary: Arc<VocabularyService>,
    pub history: Arc<HistoryService>,
    pub analytics: Arc<AnalyticsService>,
    pub files: Arc<FileJobService>,
    pub audio: Arc<AudioService>,
    pub ducking: Arc<DuckingService>,
    pub hotkeys: Arc<HotkeyService>,
    pub context: Arc<ContextService>,
    pub inference: Arc<InferenceOrchestrator>,
    pub text_inject: Arc<TextInjectService>,
    pub media: Arc<MediaService>,
    pub licence: Arc<LicenceService>,
    /// Dictation re-entry guard. `start_recording` swaps this `false → true`
    /// via a CAS; concurrent callers that lose the race bail early without
    /// arming a second capture stream. Cleared by the single successful
    /// `stop_recording` / cancel / error path that owns the active session.
    ///
    /// Before v0.6.2 a single hotkey press could fire `start_recording`
    /// three times (once per Tauri webview receiving the broadcast event),
    /// which then raced three capture streams + three recognizer builds
    /// and reliably corrupted the heap (STATUS_HEAP_CORRUPTION 0xC0000374).
    /// The frontend fan-out gate is the primary fix; this is defence in
    /// depth so a future regression can't re-introduce the same crash.
    pub recording_in_progress: AtomicBool,
    /// Transcribe-in-flight guard. Set when `stop_recording` hands audio
    /// off to the inference orchestrator, cleared when `stop_recording`
    /// returns. `start_recording` rejects while this is true — the user
    /// can't start a new dictation while the previous is mid-transcribe,
    /// because doing so races the inference write lock and (on v0.6.1)
    /// was the source of intermittent heap corruption when the two
    /// recordings competed for the DirectML session.
    ///
    /// Self-heal: if the guard has been true for more than ~30 s, it's
    /// stale (transcribe hung or dropped an error). `start_recording`
    /// sees the stale timestamp via `transcribe_started_at_ms` and
    /// clears the flag rather than wedge the user out of the app.
    pub transcribe_in_progress: AtomicBool,
    /// Unix ms of the last `transcribe_in_progress = true` transition.
    /// Used by the stale-guard self-heal in `start_recording`. Stored
    /// as an i64 atomic because that's what `now_ms()` returns; 0 means
    /// "no transcribe in flight". Writes only happen when
    /// `transcribe_in_progress` is being toggled, so no separate lock.
    pub transcribe_started_at_ms: std::sync::atomic::AtomicI64,
    handle: RwLock<Option<AppHandle>>,
}

impl AppState {
    pub fn bootstrap(data_dir: &Path) -> Result<Self> {
        let db = Arc::new(Db::open(data_dir)?);

        // Snapshot before touching schema so a broken migration is always recoverable.
        if let Err(e) = db.backup() {
            tracing::warn!(?e, "db backup failed (non-fatal)");
        }
        db.run_migrations()?;
        match db.integrity_check() {
            Ok(true) => {}
            Ok(false) => tracing::error!("sqlite integrity_check failed"),
            Err(e) => tracing::warn!(?e, "integrity_check errored"),
        }

        let settings = Arc::new(SettingsStore::load(data_dir)?);
        let modes = Arc::new(ModeService::new(db.clone()));
        modes.ensure_seed()?;

        // v0.8.0: Parakeet v2 catalogue entries retired (int8 + fp16).
        // Forced clean migration: delete any v2 model rows from the
        // installed-models table, null out modes that reference them,
        // and remove the v2 files from disk. Settings-side migration
        // (in SettingsStore::load) re-points default_asr_model_id at
        // a v3 id — but on first launch v3 isn't installed yet, so the
        // dictation path will surface the existing "no model installed"
        // UI. The user picks a v3 from Settings → Models or via the
        // onboarding "pick a model" step. Idempotent — running on an
        // already-clean DB is a no-op.
        {
            const V2_INT8: &str = "sherpa-parakeet-tdt-0.6b-v2-int8";
            const V2_FP16: &str = "sherpa-parakeet-tdt-0.6b-v2-fp16";
            if let Ok(conn) = db.conn() {
                // Modes first — null out the FK before deleting the row.
                if let Ok(n) = conn.execute(
                    "UPDATE modes SET asr_model_id = NULL WHERE asr_model_id IN (?1, ?2)",
                    rusqlite::params![V2_INT8, V2_FP16],
                ) {
                    if n > 0 {
                        tracing::warn!(rows = n, "migrated modes off retired Parakeet v2 ids");
                    }
                }
                // Read the on-disk paths before deleting so we can
                // rm them after. `path` is relative to models_root.
                let mut to_remove: Vec<String> = Vec::new();
                if let Ok(mut stmt) = conn
                    .prepare("SELECT path FROM models WHERE id IN (?1, ?2) AND path IS NOT NULL")
                {
                    let rows = stmt.query_map(rusqlite::params![V2_INT8, V2_FP16], |r| {
                        r.get::<_, String>(0)
                    });
                    if let Ok(rows) = rows {
                        for row in rows.flatten() {
                            to_remove.push(row);
                        }
                    }
                }
                if let Ok(n) = conn.execute(
                    "DELETE FROM models WHERE id IN (?1, ?2)",
                    rusqlite::params![V2_INT8, V2_FP16],
                ) {
                    if n > 0 {
                        tracing::warn!(rows = n, "removed retired Parakeet v2 model rows");
                    }
                }
                let models_root = data_dir.join("models");
                for rel in to_remove {
                    if let Some(abs) = safe_model_path(&models_root, &rel) {
                        if abs.is_dir() {
                            let _ = std::fs::remove_dir_all(&abs);
                            tracing::warn!(path = ?abs, "removed retired Parakeet v2 model directory");
                        } else if abs.is_file() {
                            let _ = std::fs::remove_file(&abs);
                        }
                    }
                }
            }
        }

        // v0.14.2: offline English Zipformer retired from the catalogue.
        // Same shape as the Parakeet v2 cleanup above: null any modes
        // referencing it, drop the models row, remove the on-disk
        // directory. Idempotent. Users keep the model file on disk only
        // if they ran a pre-v0.14.2 build with it installed; that's
        // wasted bytes nobody reads, so we reclaim it.
        {
            const ZIPFORMER_EN: &str = "sherpa-zipformer-en-int8";
            if let Ok(conn) = db.conn() {
                if let Ok(n) = conn.execute(
                    "UPDATE modes SET asr_model_id = NULL WHERE asr_model_id = ?1",
                    rusqlite::params![ZIPFORMER_EN],
                ) {
                    if n > 0 {
                        tracing::warn!(rows = n, "migrated modes off retired Zipformer id");
                    }
                }
                let mut to_remove: Vec<String> = Vec::new();
                if let Ok(mut stmt) =
                    conn.prepare("SELECT path FROM models WHERE id = ?1 AND path IS NOT NULL")
                {
                    let rows =
                        stmt.query_map(rusqlite::params![ZIPFORMER_EN], |r| r.get::<_, String>(0));
                    if let Ok(rows) = rows {
                        for row in rows.flatten() {
                            to_remove.push(row);
                        }
                    }
                }
                if let Ok(n) = conn.execute(
                    "DELETE FROM models WHERE id = ?1",
                    rusqlite::params![ZIPFORMER_EN],
                ) {
                    if n > 0 {
                        tracing::warn!(rows = n, "removed retired Zipformer model row");
                    }
                }
                let models_root = data_dir.join("models");
                for rel in to_remove {
                    if let Some(abs) = safe_model_path(&models_root, &rel) {
                        if abs.is_dir() {
                            let _ = std::fs::remove_dir_all(&abs);
                            tracing::warn!(path = ?abs, "removed retired Zipformer model directory");
                        } else if abs.is_file() {
                            let _ = std::fs::remove_file(&abs);
                        }
                    }
                }
            }
        }

        let models = Arc::new(ModelRegistry::new(db.clone(), data_dir.to_path_buf()));
        let vocabulary = Arc::new(VocabularyService::new(db.clone()));
        let history = Arc::new(HistoryService::new(db.clone()));
        let analytics = Arc::new(AnalyticsService::new(db.clone()));
        let files = Arc::new(FileJobService::new(db.clone()));
        let audio = Arc::new(AudioService::new());
        let ducking = Arc::new(DuckingService::new());
        let hotkeys = Arc::new(HotkeyService::new());
        let context = Arc::new(ContextService::new());
        let inference = Arc::new(InferenceOrchestrator::new(models.clone(), settings.clone()));
        let text_inject = Arc::new(TextInjectService::new());
        let media = Arc::new(MediaService::new());
        // Licence service computes the device fingerprint at construction time
        // (cheap, sync). The launch-time cache check runs from main.rs's setup
        // hook via `state.licence.launch_check(handle)` so it can emit events.
        let licence = LicenceService::shared();

        Ok(Self {
            db,
            settings,
            modes,
            models,
            vocabulary,
            history,
            analytics,
            files,
            audio,
            ducking,
            hotkeys,
            context,
            inference,
            text_inject,
            media,
            licence,
            recording_in_progress: AtomicBool::new(false),
            transcribe_in_progress: AtomicBool::new(false),
            transcribe_started_at_ms: std::sync::atomic::AtomicI64::new(0),
            handle: RwLock::new(None),
        })
    }

    pub async fn attach_app_handle(&self, handle: AppHandle) {
        *self.handle.write().await = Some(handle);
    }

    /// Read back the attached AppHandle. Unused today — every current background task
    /// already has a handle captured at spawn time — but kept as the canonical entry
    /// for services that only have an `Arc<AppState>` and need to emit events or reach
    /// into the window layer.
    #[allow(dead_code)]
    pub async fn app_handle(&self) -> Option<AppHandle> {
        self.handle.read().await.clone()
    }

    pub async fn is_first_run(&self) -> bool {
        !self.settings.snapshot().await.onboarding_completed
    }
}

/// Compute `models_root.join(rel)`, then check it's actually inside
/// `models_root` after canonicalisation. Returns `None` if the joined
/// path is absolute, contains `..`, or canonicalises outside the
/// models root — and logs the rejection so a corrupted DB row never
/// silently nukes something outside the models directory.
///
/// Used by the Parakeet v2 and Zipformer retirement-prune blocks
/// above. The same shape lives in `services::models::ModelRegistry::remove`
/// for active uninstalls — keeping both call sites guarded matters
/// because the prune paths run unconditionally on every app boot.
fn safe_model_path(models_root: &std::path::Path, rel: &str) -> Option<std::path::PathBuf> {
    let suspect = std::path::Path::new(rel);
    if suspect.is_absolute()
        || suspect
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        tracing::warn!(
            rel = rel,
            "model-prune refused: stored path is absolute or contains traversal"
        );
        return None;
    }
    let abs = models_root.join(suspect);
    // Empty `rel` would resolve to `models_root` itself — refuse
    // outright. The only case `assert_within` would let through here
    // is the models_root itself, which is not a model directory and
    // we never want to remove.
    if !abs.exists() {
        return None;
    }
    match crate::util::paths::assert_within(models_root, &abs) {
        Ok(canonical) if canonical != models_root.canonicalize().ok().unwrap_or_default() => {
            Some(canonical)
        }
        Ok(_) => {
            tracing::warn!(
                rel = rel,
                "model-prune refused: stored path resolves to models_root itself"
            );
            None
        }
        Err(e) => {
            tracing::warn!(
                rel = rel,
                path = %abs.display(),
                error = %e,
                "model-prune refused: target outside models_root"
            );
            None
        }
    }
}

#[cfg(test)]
mod safe_model_path_tests {
    use super::safe_model_path;
    use std::fs;

    /// A relative directory underneath `models_root` is accepted. The
    /// retirement-prune flow's happy path: a row whose `path` column
    /// is `parakeet-v2-int8` resolves to
    /// `<data_dir>/models/parakeet-v2-int8`, which exists and is inside.
    #[test]
    fn accepts_legitimate_relative_path() {
        let tmp = std::env::temp_dir().join("ut-safe-model-1");
        let _ = fs::remove_dir_all(&tmp);
        let models = tmp.join("models");
        let target = models.join("legit-id");
        fs::create_dir_all(&target).unwrap();
        let got = safe_model_path(&models, "legit-id");
        assert!(got.is_some(), "expected legit relative path to pass");
        let _ = fs::remove_dir_all(&tmp);
    }

    /// Absolute paths are rejected. Pre-fix, `models_root.join("C:\\Windows")`
    /// would have resolved to `C:\Windows` (cf. `Path::join` discards
    /// the base for absolute components) and the next `remove_dir_all`
    /// would have nuked it.
    #[test]
    fn rejects_absolute_path() {
        let tmp = std::env::temp_dir().join("ut-safe-model-2");
        let _ = fs::remove_dir_all(&tmp);
        let models = tmp.join("models");
        fs::create_dir_all(&models).unwrap();
        let abs = if cfg!(windows) { "C:\\Windows" } else { "/etc" };
        assert!(
            safe_model_path(&models, abs).is_none(),
            "expected absolute path to be rejected"
        );
        let _ = fs::remove_dir_all(&tmp);
    }

    /// `..` traversal is rejected by the explicit component scan
    /// before any filesystem access. Guards against a corrupted
    /// `path` column like `..\..\..\..\Windows\System32`.
    #[test]
    fn rejects_parent_dir_traversal() {
        let tmp = std::env::temp_dir().join("ut-safe-model-3");
        let _ = fs::remove_dir_all(&tmp);
        let models = tmp.join("models");
        fs::create_dir_all(&models).unwrap();
        assert!(
            safe_model_path(&models, "..\\escape").is_none(),
            "expected backslash traversal to be rejected"
        );
        assert!(
            safe_model_path(&models, "../escape").is_none(),
            "expected forward-slash traversal to be rejected"
        );
        let _ = fs::remove_dir_all(&tmp);
    }

    /// A relative path that doesn't exist on disk is treated as a
    /// no-op — there's nothing to delete. Returning None here means
    /// the prune block in app_state.rs simply skips it instead of
    /// trying to canonicalise a missing path (which fails on Windows).
    #[test]
    fn missing_path_returns_none() {
        let tmp = std::env::temp_dir().join("ut-safe-model-4");
        let _ = fs::remove_dir_all(&tmp);
        let models = tmp.join("models");
        fs::create_dir_all(&models).unwrap();
        assert!(
            safe_model_path(&models, "nonexistent-model-id").is_none(),
            "expected missing path to return None (no-op)"
        );
        let _ = fs::remove_dir_all(&tmp);
    }
}
