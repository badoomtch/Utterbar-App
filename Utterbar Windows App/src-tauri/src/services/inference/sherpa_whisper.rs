//! OpenAI Whisper provider backed by sherpa-onnx.
//!
//! Same async/threading model as the Parakeet provider: recogniser is held
//! in `Arc<Mutex<Option<...>>>`, all blocking work runs through
//! `tokio::task::spawn_blocking`. We never hold the mutex across an `.await`.
//!
//! Differences vs Parakeet:
//!
//! * Whisper takes a *language hint* at construction time. Real Whisper
//!   models support ~99 languages; we pass the user's `whisper_language`
//!   setting (default "auto" for runtime detection). Pinning a specific
//!   language is faster than auto-detect but loses the multilingual
//!   ability — that's a knob the user controls.
//! * The recogniser is created with `WhisperRecognizer::new` rather than
//!   `TransducerRecognizer::new`. Same DLL, different sherpa class.
//! * Whisper file naming is `tiny.en-encoder.onnx` / `tiny.en-decoder.onnx`
//!   etc. The encoder/decoder pair is paired by basename — we resolve them
//!   by suffix rather than starts-with prefix.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;

use crate::services::inference::provider::{AsrOptions, AsrProvider, AsrResult, AudioBuffer};

type Recognizer = Option<sherpa_rs::whisper::WhisperRecognizer>;

pub struct SherpaWhisper {
    model_id: String,
    model_dir: PathBuf,
    /// `auto` for runtime language detection, or an ISO-639-1 code like
    /// `en`, `fr`, `de`. Passed verbatim to sherpa-onnx; an unknown code
    /// falls back to auto on the native side.
    language: String,
    /// CPU / cuda / directml — same provider string the orchestrator
    /// resolves for Parakeet.
    provider: String,
    inner: Arc<Mutex<Recognizer>>,
}

impl SherpaWhisper {
    pub fn new(
        model_id: String,
        model_dir: PathBuf,
        language: String,
        provider: String,
    ) -> Self {
        Self {
            model_id,
            model_dir,
            language,
            provider,
            inner: Arc::new(Mutex::new(None)),
        }
    }

    /// Resolve the encoder/decoder/tokens triple from the model directory.
    ///
    /// Whisper bundles from k2-fsa use this filename shape:
    ///
    ///   <base>-encoder.onnx      (fp32)
    ///   <base>-encoder.int8.onnx (int8 quantised)
    ///   <base>-decoder.onnx
    ///   <base>-decoder.int8.onnx
    ///   <base>-tokens.txt
    ///
    /// We prefer the int8 variants because they're 3-5× smaller and
    /// noticeably faster on CPU, with little quality loss for
    /// dictation. Tokens file is `<base>-tokens.txt`, NOT plain
    /// `tokens.txt` (the original bug — the resolver assumed the
    /// Parakeet shape and returned "tokens.txt not found" on every
    /// Whisper install).
    fn resolve_files(&self) -> Result<(PathBuf, PathBuf, PathBuf)> {
        let encoder = first_with_suffix(&self.model_dir, "-encoder.int8.onnx")
            .or_else(|_| first_with_suffix(&self.model_dir, "-encoder.onnx"))?;
        let decoder = first_with_suffix(&self.model_dir, "-decoder.int8.onnx")
            .or_else(|_| first_with_suffix(&self.model_dir, "-decoder.onnx"))?;
        let tokens = first_with_suffix(&self.model_dir, "-tokens.txt")
            .or_else(|_| {
                // Fallback for bundles using the bare `tokens.txt`
                // naming (older or community builds).
                let candidate = self.model_dir.join("tokens.txt");
                if candidate.exists() {
                    Ok(candidate)
                } else {
                    Err(anyhow!(
                        "no *-tokens.txt or tokens.txt in {}",
                        self.model_dir.display()
                    ))
                }
            })?;
        Ok((encoder, decoder, tokens))
    }

    async fn ensure_loaded(&self) -> Result<()> {
        {
            let guard = self
                .inner
                .lock()
                .map_err(|_| anyhow!("recognizer mutex poisoned"))?;
            if guard.is_some() {
                tracing::info!(model = %self.model_id, "whisper::ensure_loaded cache hit");
                return Ok(());
            }
        }

        tracing::info!(model = %self.model_id, dir = %self.model_dir.display(), language = %self.language, "whisper::ensure_loaded begin");
        let (encoder, decoder, tokens) = self.resolve_files()?;
        let encoder = encoder.to_string_lossy().into_owned();
        let decoder = decoder.to_string_lossy().into_owned();
        let tokens = tokens.to_string_lossy().into_owned();

        let inner = self.inner.clone();
        let model_id = self.model_id.clone();
        // Force Whisper onto CPU regardless of the user's GPU
        // preference. Sherpa-onnx's DirectML provider has incomplete
        // op coverage for Whisper's encoder/decoder graphs — the
        // smaller `base` checkpoint mostly survives, the larger `small`
        // checkpoint reliably crashes the native layer mid-transcribe
        // (observed in the v0.8.7 logs: process died inside
        // recognizer.transcribe with no panic returned through the
        // FFI). Whisper on CPU is slower than ideal but always
        // correct. Parakeet and Zipformer keep the user's GPU
        // preference; only Whisper is force-pinned here.
        let provider = "cpu".to_string();
        let _user_pref = self.provider.clone(); // tracked for logging
        let language = self.language.clone();
        tracing::info!(
            model = %self.model_id,
            user_pref = %_user_pref,
            forced = "cpu",
            "whisper: forcing CPU provider (DirectML known-flaky on Whisper graphs)"
        );

        tokio::task::spawn_blocking(move || -> Result<()> {
            tracing::info!(%model_id, %provider, %language, "whisper: building WhisperConfig");
            let config = sherpa_rs::whisper::WhisperConfig {
                decoder,
                encoder,
                tokens,
                language,
                num_threads: Some(2),
                provider: Some(provider.clone()),
                debug: false,
                ..Default::default()
            };
            tracing::info!(%model_id, "whisper: WhisperRecognizer::new calling into native");
            let recognizer =
                sherpa_rs::whisper::WhisperRecognizer::new(config).map_err(|e| {
                    tracing::error!(%model_id, error = ?e, "whisper init failed");
                    anyhow!("whisper init: {e:?}")
                })?;
            tracing::info!(%model_id, "whisper: recognizer constructed");
            let mut guard = inner
                .lock()
                .map_err(|_| anyhow!("recognizer mutex poisoned"))?;
            *guard = Some(recognizer);
            Ok(())
        })
        .await
        .context("whisper warm spawn_blocking failed")??;
        Ok(())
    }
}

#[async_trait]
impl AsrProvider for SherpaWhisper {
    fn id(&self) -> &str {
        &self.model_id
    }

    fn supported_languages(&self) -> &[&'static str] {
        // Whisper covers ~99 languages. We don't enumerate the whole list
        // here — the language picker UI sources the catalogue's
        // `languages` field for display, and the runtime accepts any ISO
        // code Whisper recognises (it falls back to auto on unknown
        // input). Returning a representative subset is honest enough.
        &[
            "auto", "en", "fr", "de", "es", "it", "pt", "nl", "pl", "tr",
            "ja", "ko", "zh", "ru", "ar", "hi", "id", "th", "vi",
        ]
    }

    async fn warm(&self) -> Result<()> {
        self.ensure_loaded().await
    }

    async fn transcribe(&self, audio: AudioBuffer, _opts: AsrOptions) -> Result<AsrResult> {
        tracing::info!(model = %self.model_id, samples = audio.pcm.len(), sample_rate = audio.sample_rate, "whisper::transcribe enter");
        self.ensure_loaded().await?;

        let sample_rate = audio.sample_rate;
        let pcm = audio.pcm;
        let duration_ms = (pcm.len() as u64 * 1000 / sample_rate.max(1) as u64) as u32;
        let inner = self.inner.clone();
        let model_id = self.model_id.clone();
        let model_id_task = model_id.clone();
        let language_used = self.language.clone();

        let result = tokio::task::spawn_blocking(move || -> Result<(String, Option<String>)> {
            let model_id = model_id_task;
            let samples: Vec<f32> = pcm
                .into_iter()
                .map(|s| s as f32 / i16::MAX as f32)
                .collect();

            let mut guard = inner
                .lock()
                .map_err(|_| anyhow!("recognizer mutex poisoned"))?;
            let recognizer = guard
                .as_mut()
                .ok_or_else(|| anyhow!("recognizer not loaded"))?;

            if samples.is_empty() {
                tracing::warn!(%model_id, "whisper: empty audio buffer, returning empty string");
                return Ok((String::new(), None));
            }

            // Same silence + duration gates as Parakeet — Whisper's
            // hallucination behaviour on silence is famous (it
            // confabulates whole phrases of pseudo-text), so we
            // pre-reject quiet/short audio before it ever sees the
            // recognizer.
            const MIN_DURATION_MS: u32 = 180;
            const SILENCE_RMS_THRESHOLD: f32 = 0.004;
            const MARGINAL_RMS_THRESHOLD: f32 = 0.010;
            let rms_sq: f64 = samples.iter().map(|s| (*s as f64) * (*s as f64)).sum::<f64>()
                / samples.len() as f64;
            let rms = rms_sq.sqrt() as f32;
            if rms < SILENCE_RMS_THRESHOLD {
                tracing::info!(%model_id, rms, "whisper: audio below silence threshold, returning empty string");
                return Ok((String::new(), None));
            }
            if duration_ms < MIN_DURATION_MS && rms < MARGINAL_RMS_THRESHOLD {
                tracing::info!(%model_id, rms, duration_ms, "whisper: too short + too quiet, returning empty string");
                return Ok((String::new(), None));
            }

            tracing::info!(%model_id, samples = samples.len(), sample_rate, rms, "whisper: calling recognizer.transcribe (native)");
            let result = recognizer.transcribe(sample_rate, &samples);
            tracing::info!(%model_id, "whisper: recognizer.transcribe returned");

            // Whisper hallucination filter. Whisper's training data
            // included captioned video where silence/non-speech audio
            // is annotated with bracketed tags like [Music], [Applause],
            // [Background noise], (laughter), etc. When fed quiet or
            // ambiguous audio, the model confabulates the same tags as
            // the most likely "transcription" — they were in the
            // training distribution. None of them are ever a real
            // dictation result.
            //
            // The pre-recogniser RMS gate above catches obvious silence;
            // this post-recogniser filter catches the marginal cases
            // where audio cleared the gate (room tone with brief
            // movement, breath) but the model still produced a tag-
            // shaped output. Drops the whole result rather than
            // pasting [Music] into the user's app.
            let trimmed = result.text.trim();
            if is_whisper_hallucination(trimmed) {
                tracing::info!(
                    %model_id,
                    rms,
                    returned = %trimmed,
                    "whisper: dropping hallucinated bracketed-tag output"
                );
                return Ok((String::new(), None));
            }

            // sherpa_rs' WhisperRecognizerResult exposes the detected
            // language when running in auto mode. We surface it on the
            // AsrResult so downstream code (history, mode picker) can
            // record what was actually transcribed.
            let detected = if !result.lang.is_empty() {
                Some(result.lang)
            } else {
                None
            };
            Ok((result.text, detected))
        })
        .await
        .context("whisper transcribe spawn_blocking failed")?;

        let (text, detected) = result?;
        tracing::info!(
            model = %model_id,
            duration_ms,
            chars = text.len(),
            language_used = %language_used,
            detected = ?detected,
            "whisper transcribe done"
        );

        let language = detected.or_else(|| {
            if language_used == "auto" || language_used.is_empty() {
                None
            } else {
                Some(language_used)
            }
        });

        Ok(AsrResult {
            text: text.trim().to_string(),
            words: None,
            language,
            confidence: None,
            model_version: model_id,
            duration_ms,
        })
    }

    async fn unload(&self) -> Result<()> {
        if let Ok(mut guard) = self.inner.lock() {
            *guard = None;
        }
        Ok(())
    }
}

/// Detect Whisper's "captioned-video" hallucinations on quiet input.
///
/// Whisper was trained partly on subtitle/caption tracks where audio
/// without speech is annotated with bracketed tags. When you feed the
/// model silence-with-noise (room tone, breath, a chair creak), it
/// returns the most-likely caption-style annotation it saw during
/// training. None of these are ever real dictation output:
///
///   "[Music]"          "[Applause]"          "[Laughter]"
///   "[Background noise]"  "[Crowd cheering]"  "[Inaudible]"
///   "(music playing)"  "(silence)"           "♪ ♪"
///   "Thanks for watching!"   "Thank you."   "you"
///
/// Heuristic: if the trimmed output is a single short tag in brackets/
/// parentheses, OR is one of a known-hallucination phrase list, drop
/// the whole result. Conservative — we only filter outputs that match
/// these patterns AND are short. A genuine 200-word transcript that
/// happens to contain "(laughter)" mid-sentence is left alone.
fn is_whisper_hallucination(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return false;
    }

    // Single bracketed tag: "[anything]" / "(anything)" / "♪ … ♪".
    // Anchored: the WHOLE output is the tag, not embedded text.
    let bytes = trimmed.as_bytes();
    let single_bracket = (bytes.first() == Some(&b'[') && bytes.last() == Some(&b']'))
        || (bytes.first() == Some(&b'(') && bytes.last() == Some(&b')'));
    let mostly_music_glyphs = trimmed.chars().all(|c| {
        c == '♪' || c == '♫' || c == ' ' || c == '\t' || c == '.' || c == ','
    });
    if (single_bracket || mostly_music_glyphs) && trimmed.chars().count() <= 64 {
        return true;
    }

    // Stock hallucination phrases. Case-insensitive exact match on the
    // trimmed output (no punctuation stripping — Whisper's confabulated
    // outputs match their training-data form precisely).
    let normalised = trimmed.to_ascii_lowercase();
    const STOCK_PHRASES: &[&str] = &[
        "thanks for watching!",
        "thanks for watching.",
        "thanks for watching",
        "thank you.",
        "thank you",
        "thank you for watching.",
        "thank you for watching",
        "you",
        ".",
    ];
    STOCK_PHRASES.contains(&normalised.as_str())
}

fn first_with_suffix(dir: &Path, suffix: &str) -> Result<PathBuf> {
    for entry in std::fs::read_dir(dir)
        .with_context(|| format!("read_dir {}", dir.display()))?
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name.ends_with(suffix) {
            return Ok(path);
        }
    }
    Err(anyhow!("no *{suffix} file in {}", dir.display()))
}
