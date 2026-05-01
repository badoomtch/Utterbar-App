use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct Settings {
    pub schema_version: u32,
    pub onboarding_completed: bool,
    pub general: GeneralSettings,
    pub audio: AudioSettings,
    pub models: ModelSettings,
    pub text_output: TextOutputSettings,
    pub privacy: PrivacySettings,
    pub advanced: AdvancedSettings,
    pub shortcuts: ShortcutSettings,
    pub typing_baseline_wpm: u32,
    pub active_mode_id: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            schema_version: 1,
            onboarding_completed: false,
            general: GeneralSettings::default(),
            audio: AudioSettings::default(),
            models: ModelSettings::default(),
            text_output: TextOutputSettings::default(),
            privacy: PrivacySettings::default(),
            advanced: AdvancedSettings::default(),
            shortcuts: ShortcutSettings::default(),
            typing_baseline_wpm: 55,
            active_mode_id: Some("voice-to-text".into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct GeneralSettings {
    pub launch_on_login: bool,
    pub start_minimised: bool,
    pub close_to_tray: bool,
    pub check_updates_automatically: bool,
    pub theme: Theme,
    pub accent: Accent,
    pub reduce_motion: bool,
    pub sound_cues_enabled: bool,
    /// Which family of cue tones to use when `sound_cues_enabled` is true. See
    /// `SoundProfile` for the shipped profiles. Kept serde-default so older
    /// settings.json files upgrade cleanly to the `Default` profile.
    #[serde(default)]
    pub sound_profile: SoundProfile,
    /// UI scale percentage, 100 = default. Applied by the frontend by setting
    /// the root `html` element's font-size, which pulls every Tailwind `rem`
    /// unit with it. Slider on the Accessibility settings page clamps to
    /// 90..=175. Lives on GeneralSettings (rather than a separate
    /// AccessibilitySettings struct) to avoid a migration for existing users.
    #[serde(default = "default_ui_scale")]
    pub ui_scale: u32,
    /// Accessibility: boost text / border contrast everywhere. Drops mid-tone
    /// dim greys down to near-white on dark themes (and vice versa on Paper),
    /// thickens every separator by 1px, and turns "bg-1 panel on bg-0" into a
    /// visible 1px outline so low-vision users can tell where panels end.
    /// Serde-defaulted so existing settings.json files upgrade cleanly.
    #[serde(default)]
    pub high_contrast: bool,
    /// Accessibility: thicken keyboard `:focus-visible` rings from 2px to 4px
    /// and bump their opacity so tab-navigated focus is obvious on any
    /// background. No-op for mouse users (`:focus` without `:focus-visible`).
    #[serde(default)]
    pub larger_focus_rings: bool,
    /// Accessibility: disable backdrop-blur / glass effects. Keeps backgrounds
    /// solid so users with low-vision or motion-triggered discomfort don't
    /// have to parse content through a semi-transparent blur. Cheap perf win
    /// on older integrated GPUs too.
    #[serde(default)]
    pub reduce_transparency: bool,
    pub notifications_enabled: bool,
    // Bar-related preferences live on GeneralSettings for legacy reasons. They're surfaced
    // in a dedicated "Bar" settings page. All three fields default-safe via serde(default)
    // so existing settings.json files don't trip up after an upgrade.
    #[serde(default)]
    pub bar_size: BarSize,
    #[serde(default)]
    pub bar_visibility: BarVisibility,
    #[serde(default = "default_bar_outline")]
    pub bar_outline: bool,
    /// Email the user subscribed with for release-announcement mail, if any.
    /// `None` means the user hasn't opted in. When `Some`, the Settings →
    /// General UI shows a "subscribed" state. We keep the email locally so the
    /// toggle can round-trip across restarts without re-prompting; the server
    /// is the authoritative list (via `update_subscribers` + confirmation mail)
    /// and one-click unsubscribe lives on the server too. Serde-defaulted so
    /// older settings.json files upgrade cleanly.
    #[serde(default)]
    pub releases_email: Option<String>,
    /// Hex string (e.g. `"#22D3EE"`) backing the `Accent::Custom` variant.
    /// The frontend's theme applier reads this when `accent == Custom` and
    /// sets `--accent` / `--accent-2` / `--accent-3` from it. Stored
    /// regardless of current tier so a Free user who ever had Pro gets
    /// their colour back when they re-activate. Validated loosely — we
    /// accept any 7-char `#RRGGBB` string and fall back to cyan at render
    /// time if it's missing or malformed.
    #[serde(default)]
    pub custom_accent_hex: Option<String>,
}

fn default_bar_outline() -> bool {
    true
}

fn default_ui_scale() -> u32 {
    100
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
#[serde(rename_all = "snake_case")]
pub enum BarSize {
    Compact,
    #[default]
    Standard,
}

/// Cue-sound flavour. All profiles are synthesised on the fly via the frontend's
/// Web Audio API — we keep the enum in Rust so the choice survives a restart
/// and round-trips through settings.json. Extending the family means adding a
/// variant here and a matching entry in `src/design/sounds.ts`.
///
/// * `Default` — short two-note rising/falling sines, the original Utterbar cues.
/// * `Classic` — flat dual-square-wave beeps, reminiscent of a 90s voicemail.
/// * `Subtle` — single very-soft low sine, for users who want a near-imperceptible nudge.
/// * `Chime` — triangle-wave thirds with longer decay; more bell-like and calmer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
#[serde(rename_all = "snake_case")]
pub enum SoundProfile {
    #[default]
    Default,
    Classic,
    Subtle,
    Chime,
}

/// When the floating Utterbar is shown on screen.
/// * `Always` — pill is always visible (default).
/// * `WhileRecording` — pill appears only during active dictation phases.
/// * `Hidden` — pill never appears; the user drives everything from the tray + shortcuts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
#[serde(rename_all = "snake_case")]
pub enum BarVisibility {
    #[default]
    Always,
    WhileRecording,
    Hidden,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            launch_on_login: false,
            start_minimised: true,
            close_to_tray: true,
            check_updates_automatically: true,
            theme: Theme::Obsidian,
            accent: Accent::Teal,
            reduce_motion: false,
            sound_cues_enabled: false,
            sound_profile: SoundProfile::default(),
            ui_scale: default_ui_scale(),
            high_contrast: false,
            larger_focus_rings: false,
            reduce_transparency: false,
            notifications_enabled: true,
            bar_size: BarSize::Standard,
            bar_visibility: BarVisibility::Always,
            bar_outline: true,
            releases_email: None,
            custom_accent_hex: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    Obsidian,
    Graphite,
    Paper,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
#[serde(rename_all = "snake_case")]
pub enum Accent {
    Teal,
    Amber,
    Violet,
    Rose,
    Cyan,
    Lime,
    /// User-picked hex accent. Read from `GeneralSettings.custom_accent_hex`
    /// at render time. Pro-only — the backend strips the variant back to the
    /// default Teal for Free tier in `update_settings` (see `commands/
    /// settings.rs`), same shape as the existing `retain_audio` strip. This
    /// variant is deliberately not a tuple variant holding the hex itself:
    /// keeping the colour on `GeneralSettings` means the user's last pick
    /// sticks around even if they flip back to a preset and return later.
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct AudioSettings {
    pub input_device_id: Option<String>,
    pub output_device_id: Option<String>,
    pub dynamic_normalisation: bool,
    pub silence_removal: bool,
    /// Strength of the inline noise suppressor. Older settings.json files
    /// stored this as a bool — the custom deserializer below maps
    /// `false`→`Off` and `true`→`Balanced`, so no migration is required on
    /// disk. Users can pick a level explicitly from Audio settings.
    #[serde(default, deserialize_with = "de_noise_suppression")]
    pub noise_suppression: NoiseSuppression,
    /// Noise-gate aggression, independent of the broadband suppressor. When
    /// on, short sub-threshold blips between words are zeroed so the ASR
    /// isn't fed keyboard clicks / lip-smacks / breath sounds. Off by
    /// default because an over-tight gate can clip the tail of soft
    /// consonants.
    #[serde(default)]
    pub noise_gate: bool,
    pub input_gain_db: f32,
    /// What to record: the mic, the system's playback (WASAPI loopback), or both mixed.
    /// Defaults to `Microphone` so existing installs continue to behave identically.
    #[serde(default)]
    pub capture_source: CaptureSource,
    /// Diagnostic mode: write the raw captured PCM and the post-preprocess
    /// PCM (the exact buffer handed to the ASR) as WAVs under
    /// `%APPDATA%\Utterbar\diagnostics\audio\`. Off by default — only
    /// enable when reproducing an audio-quality bug, since it leaks
    /// private audio to disk. Files self-rotate (the latest pair plus
    /// at most a few historical pairs are kept) so it doesn't fill the
    /// drive even if left on.
    #[serde(default)]
    pub diagnostic_audio_dump: bool,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            input_device_id: None,
            output_device_id: None,
            dynamic_normalisation: false,
            silence_removal: false,
            noise_suppression: NoiseSuppression::Off,
            noise_gate: false,
            input_gain_db: 0.0,
            capture_source: CaptureSource::Microphone,
            diagnostic_audio_dump: false,
        }
    }
}

/// Strength of the DC-blocker + high-pass + adaptive noise-floor suppressor.
/// All levels run in well under a millisecond — the knob trades artefact
/// risk for quietness.
///
/// * `Off` — pass the mic through as-is. Lowest latency, preserves every
///   subtle cue for the ASR.
/// * `Light` — cut DC offset and rumble below ~60 Hz. Leaves mid-band hiss
///   alone. Recommended for quiet rooms where you just want to strip
///   subwoofer-band garbage.
/// * `Balanced` — the old "on" default. Kills fan hum, HVAC, and keyboard
///   clatter between words. Best for everyday office / home use.
/// * `Aggressive` — pushes the gate threshold higher and the high-pass
///   corner up to ~140 Hz. Best for cafés, open-plan offices, and
///   roommate-shaped problems. Occasionally clips the tail of soft
///   consonants ("s", "f") so only use it when a Light/Balanced setting
///   isn't cutting through the room.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
#[serde(rename_all = "snake_case")]
pub enum NoiseSuppression {
    #[default]
    Off,
    Light,
    Balanced,
    Aggressive,
}

/// Accept the old `bool` shape as well as the new string shape so existing
/// settings.json files keep loading. `true` maps to `Balanced` (the
/// behaviour the bool was actually delivering) and `false` maps to `Off`.
fn de_noise_suppression<'de, D>(deserializer: D) -> Result<NoiseSuppression, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let v = serde_json::Value::deserialize(deserializer)?;
    match v {
        serde_json::Value::Bool(false) => Ok(NoiseSuppression::Off),
        serde_json::Value::Bool(true) => Ok(NoiseSuppression::Balanced),
        serde_json::Value::String(s) => match s.as_str() {
            "off" => Ok(NoiseSuppression::Off),
            "light" => Ok(NoiseSuppression::Light),
            "balanced" => Ok(NoiseSuppression::Balanced),
            "aggressive" => Ok(NoiseSuppression::Aggressive),
            other => Err(D::Error::custom(format!(
                "unknown noise_suppression level '{other}'"
            ))),
        },
        _ => Err(D::Error::custom(
            "noise_suppression must be a bool or a level string",
        )),
    }
}

/// Which audio stream Utterbar should transcribe.
/// * `Microphone` — physical mic (default, unchanged behaviour).
/// * `System` — everything coming out of the speakers via WASAPI loopback. Useful for
///   transcribing a meeting or a video that's already playing in another app.
/// * `Mixed` — both mic and system audio summed into a single mono stream, so a
///   conversation where *you* are also speaking gets both sides.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
#[serde(rename_all = "snake_case")]
pub enum CaptureSource {
    #[default]
    Microphone,
    System,
    Mixed,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct ModelSettings {
    pub default_asr_model_id: Option<String>,
    pub default_llm_model_id: Option<String>,
    pub warm_duration_secs: u32,
    pub experimental_models: bool,
    /// ISO-639-1 code (or "auto") passed to Whisper recognisers for
    /// language hinting. Ignored by Parakeet / Zipformer (English-only
    /// engines). Default `auto` lets Whisper detect the language on
    /// each utterance — slightly slower than a pinned language but
    /// the right behaviour when the user dictates in more than one.
    /// Examples: "en", "fr", "de", "es", "ja". Empty string is
    /// equivalent to "auto" for back-compat with older settings.json.
    #[serde(default = "default_whisper_language")]
    pub whisper_language: String,
    /// Meeting-mode ASR model preference (v0.15.3+). When `Some(id)`
    /// the dictation flow resolves the meeting model to this id and
    /// refuses start if the model isn't installed. When `None` the
    /// flow prefers Whisper Medium if installed and refuses
    /// otherwise (no silent Large/Small/Parakeet fallback). Distinct
    /// from `default_asr_model_id`, which only governs non-meeting
    /// modes. Pre-v0.15.3 settings.json files miss this field
    /// entirely; serde `default` deserialises them with `None`.
    #[serde(default)]
    pub meeting_asr_model_id: Option<String>,
}

fn default_whisper_language() -> String {
    "auto".into()
}

impl Default for ModelSettings {
    fn default() -> Self {
        Self {
            default_asr_model_id: None,
            default_llm_model_id: None,
            warm_duration_secs: 120,
            experimental_models: false,
            whisper_language: "auto".into(),
            meeting_asr_model_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct TextOutputSettings {
    pub paste_automatically: bool,
    pub restore_clipboard: bool,
    pub keystroke_fallback: bool,
    pub shift_to_auto_send: bool,
    pub auto_capitalise: bool,
    pub trim_trailing_space: bool,
    /// When true, and the user fires a follow-up dictation in the same app shortly after
    /// a previous one that didn't end with whitespace, prepend a space so "hello" +
    /// "world" becomes "hello world" instead of "helloworld". On by default — the
    /// heuristic is conservative (same-app + short time window) so false positives are
    /// rare.
    #[serde(default = "default_auto_space_on_paste")]
    pub auto_space_on_paste: bool,
    /// When true, post-process the transcript to convert spelled-out
    /// numbers ("twenty three") into digits ("23"). Most ASR models
    /// emit words by default; users dictating prices, dates, code,
    /// addresses etc. usually want digits. On by default — the
    /// transform is conservative (only converts unambiguous numeric
    /// phrases, leaves "one of these days" alone).
    #[serde(default = "default_numbers_as_digits")]
    pub numbers_as_digits: bool,
    /// When true, run the existing deterministic rewrite pass on
    /// regular dictation output globally, regardless of the active
    /// mode's `post_processing` setting. The rewrite removes filler
    /// words ("um", "uh", "like"), normalises whitespace, ensures
    /// terminal punctuation, and capitalises sentence starts. All
    /// transformations are local and rule-based — no LLM, no model
    /// download, no remote calls.
    ///
    /// Default off because most users prefer the verbatim ASR output
    /// (already capitalised correctly by Whisper / Parakeet on most
    /// inputs) and filler removal is opinionated. Meeting Mode is
    /// deliberately out of scope: meeting transcripts are reference
    /// material where verbatim fidelity matters more than polish.
    /// `serde(default)` so settings.json files written before this
    /// field existed deserialise cleanly with `false`.
    #[serde(default)]
    pub smart_rewrite: bool,
}

fn default_auto_space_on_paste() -> bool {
    true
}

fn default_numbers_as_digits() -> bool {
    true
}

impl Default for TextOutputSettings {
    fn default() -> Self {
        Self {
            paste_automatically: true,
            restore_clipboard: true,
            keystroke_fallback: true,
            shift_to_auto_send: false,
            auto_capitalise: true,
            trim_trailing_space: true,
            auto_space_on_paste: true,
            numbers_as_digits: true,
            smart_rewrite: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct PrivacySettings {
    pub retain_audio: bool,
    pub retain_audio_days: u32,
    pub allow_context_clipboard: bool,
    pub allow_context_selection: bool,
    pub allow_context_active_app: bool,
    /// Automatically pause Spotify / YouTube / any SMTC-registered media while
    /// recording, then resume only the sessions we paused when the recording ends.
    pub pause_media_while_recording: bool,
    /// Mute the output (what you HEAR) of Teams / Slack / Discord / Zoom / Webex
    /// while a recording is active, so incoming call audio and notification dings
    /// don't distract from dictation. Restored the instant recording stops.
    ///
    /// Off by default — this touches the volume of apps other than Utterbar, which
    /// is an opt-in choice. Does NOT mute your microphone to those apps; use each
    /// app's own mute button for that (a reliable cross-app mute requires per-app
    /// shortcut simulation, which is brittle and deliberately out of scope here).
    #[serde(default)]
    pub mute_conferencing_while_recording: bool,
    /// Override list of EXE basenames to mute when `mute_conferencing_while_recording`
    /// is on. When empty, falls back to `DEFAULT_CONFERENCING_APPS` in the ducking
    /// service. Case-insensitive match. Advanced users only — most people should
    /// leave this empty and trust the defaults.
    #[serde(default)]
    pub conferencing_apps_override: Vec<String>,
}

impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            retain_audio: false,
            retain_audio_days: 14,
            allow_context_clipboard: false,
            allow_context_selection: false,
            allow_context_active_app: true,
            pause_media_while_recording: true,
            mute_conferencing_while_recording: false,
            conferencing_apps_override: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct AdvancedSettings {
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default)]
    pub gpu_acceleration: GpuAcceleration,
    #[serde(default)]
    pub thread_count: Option<u32>,
    /// Update channel for the auto-updater. `stable` is the default and
    /// what the public sees. `beta` opts into pre-release builds —
    /// passed as `?channel=beta` to `/api/updater/check`. Useful for
    /// the developer dogfooding fixes before publishing them stable.
    /// Anything other than `beta` is treated as `stable` server-side.
    #[serde(default = "default_update_channel")]
    pub update_channel: String,
}

fn default_log_level() -> String {
    "info".into()
}

fn default_update_channel() -> String {
    "stable".into()
}

impl Default for AdvancedSettings {
    fn default() -> Self {
        Self {
            log_level: "info".into(),
            // `Auto` means "use the GPU if one is detected, otherwise fall back to CPU".
            // The inference layer resolves this to a concrete provider at load time.
            gpu_acceleration: GpuAcceleration::Auto,
            thread_count: None,
            update_channel: "stable".into(),
        }
    }
}

/// User-facing GPU acceleration preference. Resolved to a concrete ONNX Runtime
/// provider string (`"cpu"` / `"directml"` / `"cuda"`) by the inference layer,
/// taking the detected GPU hardware into account.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
#[serde(rename_all = "snake_case")]
pub enum GpuAcceleration {
    /// Use the GPU if one is present, otherwise CPU. Recommended default.
    #[default]
    Auto,
    /// Force DirectML (Windows, any DX12 GPU).
    DirectMl,
    /// Force NVIDIA CUDA.
    Cuda,
    /// Force CPU — useful when the user wants to keep the GPU free for other apps.
    Cpu,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct ShortcutSettings {
    pub toggle_recording: Option<crate::domain::Shortcut>,
    pub push_to_talk: Option<crate::domain::Shortcut>,
    pub hold_to_talk: Option<crate::domain::Shortcut>,
    pub cancel_recording: Option<crate::domain::Shortcut>,
    pub cycle_mode: Option<crate::domain::Shortcut>,
    pub open_settings: Option<crate::domain::Shortcut>,
    pub open_history: Option<crate::domain::Shortcut>,
    /// Add-to-vocabulary shortcut. Defaults to None (unbound) — no
    /// accelerator we'd pick is reliably free across apps, so we ask the
    /// user to choose one from Settings → Shortcuts. The Vocabulary page
    /// surfaces a "Bind a shortcut" CTA when this is None.
    #[serde(default)]
    pub add_to_vocabulary: Option<crate::domain::Shortcut>,
}

impl Default for ShortcutSettings {
    fn default() -> Self {
        use crate::domain::{Shortcut, TriggerStyle};
        // Defaults chosen to minimise collisions with common Windows and app bindings.
        // Notes on the choices:
        // * Ctrl+Space is the conventional "start dictation" shortcut across
        //   dictation tools (Windows Speech, Dragon, Wispr Flow), so we keep it.
        // * Ctrl+Alt+<letter/symbol> is the safest prefix on Windows: not claimed
        //   by the shell, not claimed by Office (which prefers Ctrl+Shift), not
        //   the Ctrl+Shift prefix that IDEs rely on. AltGr on non-US keyboards
        //   maps to Ctrl+Alt and can produce glyphs, but Utterbar is Windows-only
        //   and the user can rebind from Settings → Shortcuts if a binding
        //   conflicts with their layout.
        // * F-keys are reserved as a last-resort fallback — hold-to-talk uses
        //   F6 because it's the least-claimed of F2-F10 on Windows.
        Self {
            toggle_recording: Some(Shortcut {
                accelerator: "Control+Space".into(),
                label: "Ctrl+Space".into(),
                trigger_style: TriggerStyle::Toggle,
            }),
            push_to_talk: Some(Shortcut {
                accelerator: "Control+Alt+Space".into(),
                label: "Ctrl+Alt+Space".into(),
                trigger_style: TriggerStyle::PushToTalk,
            }),
            hold_to_talk: Some(Shortcut {
                accelerator: "F6".into(),
                label: "F6".into(),
                trigger_style: TriggerStyle::HoldToTalk,
            }),
            cancel_recording: Some(Shortcut {
                accelerator: "Control+Alt+X".into(),
                label: "Ctrl+Alt+X".into(),
                trigger_style: TriggerStyle::Toggle,
            }),
            cycle_mode: Some(Shortcut {
                accelerator: "Control+Alt+M".into(),
                label: "Ctrl+Alt+M".into(),
                trigger_style: TriggerStyle::Toggle,
            }),
            open_settings: Some(Shortcut {
                accelerator: "Control+Alt+Comma".into(),
                label: "Ctrl+Alt+,".into(),
                trigger_style: TriggerStyle::Toggle,
            }),
            open_history: Some(Shortcut {
                accelerator: "Control+Alt+H".into(),
                label: "Ctrl+Alt+H".into(),
                trigger_style: TriggerStyle::Toggle,
            }),
            // No safe default — see field doc. User picks one in Settings.
            add_to_vocabulary: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct DataPaths {
    pub config: String,
    pub data: String,
    pub models: String,
    pub recordings: String,
    pub logs: String,
}

#[cfg(test)]
mod text_output_tests {
    //! Pin the v1 contract for `text_output.smart_rewrite`:
    //!   1. Default is `false` so users on existing installs see no
    //!      behaviour change after upgrade.
    //!   2. `serde(default)` covers settings.json files written
    //!      before this field existed — they must deserialise to
    //!      `false` cleanly, no migration step needed.
    //!
    //! These guards are cheap and would fail loudly if anyone ever
    //! flipped the default to `true` without rewriting the privacy /
    //! marketing copy that calls Utterbar "verbatim, no rewrites by
    //! default". The deserialise test is the load-bearing one for
    //! upgrade safety.

    use super::*;

    #[test]
    fn smart_rewrite_default_is_false() {
        let t = TextOutputSettings::default();
        assert!(
            !t.smart_rewrite,
            "smart_rewrite must default to false; flipping it to true would silently \
             change every existing user's transcripts on next launch and contradict \
             the privacy positioning."
        );
    }

    #[test]
    fn settings_default_smart_rewrite_is_false() {
        // Same guard via the parent Settings::default() path that
        // get_settings actually returns when settings.json is missing.
        let s = Settings::default();
        assert!(!s.text_output.smart_rewrite);
    }

    #[test]
    fn pre_smart_rewrite_settings_json_deserialises_with_false() {
        // The exact shape of TextOutputSettings as it shipped in
        // v0.16.3 (before this field existed). settings.json files
        // written by that build must continue to load cleanly with
        // smart_rewrite = false. If a future change drops the
        // serde(default) attribute or makes the field non-optional,
        // every existing settings.json file fails to parse and the
        // whole settings layer falls back to defaults — losing the
        // user's saved preferences.
        let pre_v1_json = r#"{
            "paste_automatically": true,
            "restore_clipboard": true,
            "keystroke_fallback": true,
            "shift_to_auto_send": false,
            "auto_capitalise": true,
            "trim_trailing_space": true,
            "auto_space_on_paste": true,
            "numbers_as_digits": true
        }"#;
        let parsed: TextOutputSettings =
            serde_json::from_str(pre_v1_json).expect("pre-v1 settings.json must still deserialise");
        assert!(
            !parsed.smart_rewrite,
            "missing smart_rewrite field must deserialise to false (default)"
        );
        // Sanity: existing fields still round-trip.
        assert!(parsed.paste_automatically);
        assert!(parsed.numbers_as_digits);
    }

    #[test]
    fn post_v1_settings_json_round_trips_smart_rewrite_value() {
        let t = TextOutputSettings {
            smart_rewrite: true,
            ..TextOutputSettings::default()
        };
        let json = serde_json::to_string(&t).expect("serialise");
        let parsed: TextOutputSettings = serde_json::from_str(&json).expect("deserialise");
        assert!(parsed.smart_rewrite, "round-trip must preserve true");
    }
}
