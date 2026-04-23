//! Central dependency container. Everything touching long-lived state lives on AppState.

use std::path::Path;
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

        // fp16-on-CPU crash migration, DB side: null out any mode.asr_model_id that points
        // at the removed fp16 Parakeet variant. Settings.json was handled in SettingsStore.
        {
            const FP16_ID: &str = "sherpa-parakeet-tdt-0.6b-v2-fp16";
            if let Ok(conn) = db.conn() {
                if let Ok(n) = conn.execute(
                    "UPDATE modes SET asr_model_id = NULL WHERE asr_model_id = ?1",
                    rusqlite::params![FP16_ID],
                ) {
                    if n > 0 {
                        tracing::warn!(rows = n, "migrated modes off fp16 Parakeet model");
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
