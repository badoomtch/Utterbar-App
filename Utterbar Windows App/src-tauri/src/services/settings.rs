//! JSON-backed settings store with atomic writes and in-memory cache.
//!
//! Storage location: `%APPDATA%\Utterbar\config\settings.json` (derived from
//! `util::paths::app_data_dir()` on Windows). This **must** stay outside the
//! install directory so user preferences survive an NSIS upgrade — the
//! installer wipes `C:\Program Files\Utterbar` but leaves `%APPDATA%` alone,
//! which is exactly what we want. If you ever refactor the path choice, keep
//! it rooted at `%APPDATA%`; do NOT use `resolve_path(.., AppLocalData)` or
//! anything under the install dir, and do NOT cache to a temp folder that
//! gets GC'd on boot. Licence is separately safe — see `services/licence/
//! cache.rs`, which uses the Windows Credential Manager (DPAPI), a
//! process-independent store.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use tokio::sync::RwLock;

use crate::domain::Settings;

pub struct SettingsStore {
    path: PathBuf,
    cache: RwLock<Settings>,
}

impl SettingsStore {
    pub fn load(data_dir: &Path) -> Result<Self> {
        let config = data_dir.join("config");
        std::fs::create_dir_all(&config)?;
        let path = config.join("settings.json");

        let mut cache = if path.exists() {
            let raw = std::fs::read_to_string(&path).context("read settings.json")?;
            // Fall back to defaults on corruption rather than crashing the app.
            serde_json::from_str::<Settings>(&raw).unwrap_or_default()
        } else {
            let s = Settings::default();
            persist(&path, &s)?;
            s
        };

        // Back-compat migration: early 0.3.x builds exposed a Parakeet fp16 variant that
        // crashes the CPU ONNX runtime with std::terminate. We now ship an fp16 entry again
        // because GPU acceleration is plumbed, but only auto-migrate the reference away
        // when the user has explicitly forced CPU — otherwise the inference layer will
        // fail gracefully with a user-facing error instead of a crash.
        let migrated = migrate_fp16_reference_if_cpu_forced(&mut cache);
        if migrated {
            tracing::warn!("migrated default_asr_model_id away from fp16 variant (CPU forced)");
            let _ = persist(&path, &cache);
        }

        // v0.8.0: Parakeet v2 (int8 + fp16) dropped from the catalogue —
        // v3 supersedes it. Any user whose default_asr_model_id still
        // points at the old v2 ids gets transparently moved to the v3
        // int8 default. v2 model files on disk are NOT deleted (the
        // user paid the download cost; we leave the bytes alone) — the
        // catalogue just stops listing v2 as installable. If the user
        // has no v3 model installed yet, the dictation path will surface
        // the standard "no model installed" UI on next use.
        let migrated_v2 = migrate_v2_references_to_v3(&mut cache);
        if migrated_v2 {
            tracing::warn!("migrated default_asr_model_id from Parakeet v2 to v3");
            let _ = persist(&path, &cache);
        }

        // Back-compat migration: early builds shipped before some shortcut slots
        // existed (cancel/cycle/settings/history) or with several slots unbound.
        // New installs get all defaults via `ShortcutSettings::default()`, but
        // existing users upgrading from 0.3.x end up with `None` for anything
        // that wasn't in their saved settings.json — which reads as "Utterbar
        // didn't ship any keybinds", even though the Rust Default impl has them.
        // Backfill any empty slot from defaults so an upgrade feels identical to
        // a fresh install for keybinds the user never explicitly touched.
        let backfilled = backfill_shortcut_defaults(&mut cache);
        if backfilled {
            tracing::info!("backfilled unbound shortcut slots from defaults");
            let _ = persist(&path, &cache);
        }

        Ok(Self {
            path,
            cache: RwLock::new(cache),
        })
    }

    pub async fn snapshot(&self) -> Settings {
        self.cache.read().await.clone()
    }

    pub async fn update<F>(&self, mutator: F) -> Result<Settings>
    where
        F: FnOnce(&mut Settings),
    {
        let mut guard = self.cache.write().await;
        mutator(&mut guard);
        persist(&self.path, &guard)?;
        Ok(guard.clone())
    }

    pub async fn reset(&self) -> Result<Settings> {
        let mut guard = self.cache.write().await;
        *guard = Settings::default();
        persist(&self.path, &guard)?;
        Ok(guard.clone())
    }
}

fn migrate_fp16_reference_if_cpu_forced(s: &mut Settings) -> bool {
    use crate::domain::settings::GpuAcceleration;
    const FP16_ID: &str = "sherpa-parakeet-tdt-0.6b-v2-fp16";
    const INT8_ID: &str = "sherpa-parakeet-tdt-0.6b-v2-int8";

    if s.advanced.gpu_acceleration != GpuAcceleration::Cpu {
        return false;
    }

    let mut changed = false;
    if s.models.default_asr_model_id.as_deref() == Some(FP16_ID) {
        s.models.default_asr_model_id = Some(INT8_ID.into());
        changed = true;
    }
    changed
}

/// v0.8.0: Parakeet v2 catalogue entries were retired. Clear any
/// setting that still references a v2 id so the next dictation
/// surfaces the existing "no model installed" prompt and the user
/// picks a v3 from Settings → Models. We deliberately do NOT
/// auto-redirect to a v3 id — the v3 model isn't installed on disk,
/// so pointing at it would produce a confusing runtime error
/// ("model not installed: …-v3-int8") instead of the clean
/// pick-a-model flow. Idempotent.
fn migrate_v2_references_to_v3(s: &mut Settings) -> bool {
    const V2_INT8: &str = "sherpa-parakeet-tdt-0.6b-v2-int8";
    const V2_FP16: &str = "sherpa-parakeet-tdt-0.6b-v2-fp16";

    if matches!(
        s.models.default_asr_model_id.as_deref(),
        Some(V2_INT8) | Some(V2_FP16)
    ) {
        s.models.default_asr_model_id = None;
        return true;
    }
    false
}

/// Fill any `None` shortcut slot from `ShortcutSettings::default()`. Non-None
/// slots (explicit rebinds, or explicit Nones if the user cleared a binding)
/// are left alone — we only touch fields the old settings.json literally didn't
/// know about, or where the user never exercised the bind UI.
///
/// Returns `true` when at least one slot was written.
fn backfill_shortcut_defaults(s: &mut Settings) -> bool {
    let defaults = crate::domain::settings::ShortcutSettings::default();
    let sh = &mut s.shortcuts;
    let mut changed = false;
    macro_rules! fill {
        ($field:ident) => {
            if sh.$field.is_none() && defaults.$field.is_some() {
                sh.$field = defaults.$field.clone();
                changed = true;
            }
        };
    }
    fill!(toggle_recording);
    fill!(push_to_talk);
    fill!(hold_to_talk);
    fill!(cancel_recording);
    fill!(cycle_mode);
    fill!(open_settings);
    fill!(open_history);
    changed
}

fn persist(path: &Path, s: &Settings) -> Result<()> {
    let tmp = path.with_extension("json.tmp");
    let json = serde_json::to_string_pretty(s)?;
    std::fs::write(&tmp, json)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}
