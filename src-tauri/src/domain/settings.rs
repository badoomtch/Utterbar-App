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
}

impl Default for ModelSettings {
    fn default() -> Self {
        Self {
            default_asr_model_id: None,
            default_llm_model_id: None,
            warm_duration_secs: 120,
            experimental_models: false,
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
}

fn default_auto_space_on_paste() -> bool {
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
}

fn default_log_level() -> String {
    "info".into()
}

impl Default for AdvancedSettings {
    fn default() -> Self {
        Self {
            log_level: "info".into(),
            // `Auto` means "use the GPU if one is detected, otherwise fall back to CPU".
            // The inference layer resolves this to a concrete provider at load time.
            gpu_acceleration: GpuAcceleration::Auto,
            thread_count: None,
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
