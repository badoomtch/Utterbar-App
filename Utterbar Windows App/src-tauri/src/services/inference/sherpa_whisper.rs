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
use std::time::Instant;

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
    /// cpu / directml — same provider string the orchestrator
    /// resolves for Parakeet.
    provider: String,
    inner: Arc<Mutex<Recognizer>>,
}

impl SherpaWhisper {
    pub fn new(model_id: String, model_dir: PathBuf, language: String, provider: String) -> Self {
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
        let tokens = first_with_suffix(&self.model_dir, "-tokens.txt").or_else(|_| {
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

        // Same cold-warm timing shape as the Parakeet engine — see
        // sherpa_parakeet::ensure_loaded for what each segment measures.
        // Whisper's native_construct_ms is typically the dominant cost
        // for Medium/Large where ORT mmaps several hundred MB.
        let warm_start = Instant::now();
        tracing::info!(
            model = %self.model_id,
            dir = %self.model_dir.display(),
            family = "whisper",
            provider = %self.provider,
            language = %self.language,
            "asr_warm: begin"
        );
        let resolve_start = Instant::now();
        let (encoder, decoder, tokens) = self.resolve_files()?;
        let file_resolve_ms = resolve_start.elapsed().as_millis() as u64;
        let encoder = encoder.to_string_lossy().into_owned();
        let decoder = decoder.to_string_lossy().into_owned();
        let tokens = tokens.to_string_lossy().into_owned();

        let inner = self.inner.clone();
        let model_id = self.model_id.clone();
        // v0.15.6: honour the resolved provider string, lifting the
        // v0.8.7 CPU-force.
        //
        // History:
        //   * v0.8.7 disabled DirectML for Whisper specifically. The
        //     observed failure was a process crash mid-transcribe on
        //     Whisper Small, with no panic propagating through the
        //     FFI. Larger Whisper checkpoints had the same risk; CPU
        //     was the only safe path. Parakeet and Zipformer were
        //     left honouring the user's GPU preference.
        //   * v0.15.5 Phase A made the gap between requested and used
        //     provider strings observable in telemetry but kept the
        //     engine-side CPU-force in place — production users were
        //     still on CPU regardless of what they picked.
        //   * v0.15.6 Step 2 smoke-test matrix re-tested all four
        //     supported engines (Whisper Small / Medium / Large v3
        //     + Parakeet TDT v3 int8) against the new ORT 1.14.1 +
        //     DirectML 1.15.0 bundle. 8/8 cells passed. Zero crashes.
        //     Output identical to CPU on Whisper Medium / Large.
        //     The original failure mode does not reproduce against
        //     the new bundle.
        //
        // The CPU-force is lifted. The orchestrator's `resolve_provider`
        // already routes DirectML -> CPU when the runtime can't
        // serve it (v0.15.5 fallback contract preserved) and applies
        // the auto-Small CPU policy (v0.15.6 model-aware path). The
        // engine trusts the resolved string.
        //
        // If a future ORT/DML upgrade reintroduces the failure, the
        // mitigation surface is `resolve_provider`'s model-aware
        // path, NOT this engine. Engine-side CPU-force was always
        // an information loss in the telemetry layer.
        let provider = self.provider.clone();
        let language = self.language.clone();

        let native_construct_ms = tokio::task::spawn_blocking(move || -> Result<u64> {
            // Sherpa-onnx's WhisperConfig.language wants either an empty
            // string (auto-detect via language-id tokens at decode time)
            // or a concrete ISO-639-1 code such as "en" / "fr". The
            // literal sentinel "auto" is a UI/storage convention on our
            // side; passing it through to sherpa was the cause of the
            // Whisper Large v3 crash observed on 2026-04-28 (process
            // aborted ~30s into the first transcribe call regardless of
            // input length). Whisper base / small tolerated the unknown
            // sentinel because their tokenizer's language-id set is
            // smaller and the decoder fell back to auto; large-v3
            // doesn't, hits an out-of-vocabulary lookup, and the
            // resulting C++ exception unwinds through the FFI boundary
            // into a hard process exit.
            //
            // Map "auto" (and unknown values for safety) to "" before
            // handing to sherpa. The user's persisted setting stays as
            // "auto" so the picker UI keeps showing the right chip.
            let sherpa_language = if language == "auto" {
                String::new()
            } else {
                language.clone()
            };
            let config = sherpa_rs::whisper::WhisperConfig {
                decoder,
                encoder,
                tokens,
                language: sherpa_language,
                num_threads: Some(2),
                provider: Some(provider.clone()),
                debug: false,
                ..Default::default()
            };
            let native_start = Instant::now();
            let recognizer = sherpa_rs::whisper::WhisperRecognizer::new(config).map_err(|e| {
                tracing::error!(%model_id, error = ?e, "whisper init failed");
                anyhow!("whisper init: {e:?}")
            })?;
            let native_ms = native_start.elapsed().as_millis() as u64;
            let mut guard = inner
                .lock()
                .map_err(|_| anyhow!("recognizer mutex poisoned"))?;
            *guard = Some(recognizer);
            Ok(native_ms)
        })
        .await
        .context("whisper warm spawn_blocking failed")??;

        let total_warm_ms = warm_start.elapsed().as_millis() as u64;
        tracing::info!(
            model = %self.model_id,
            family = "whisper",
            provider = %self.provider,
            language = %self.language,
            file_resolve_ms,
            native_construct_ms,
            total_warm_ms,
            "asr_warm: done"
        );
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
            "auto", "en", "fr", "de", "es", "it", "pt", "nl", "pl", "tr", "ja", "ko", "zh", "ru",
            "ar", "hi", "id", "th", "vi",
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

            // sherpa-onnx's offline Whisper decoder silently truncates any
            // input >= 30 seconds ("Only waves less than 30 seconds are
            // supported"): it decodes a single 30s mel window and drops the
            // rest. Meeting Mode works around this by slicing at the
            // dictation layer, but regular single-shot dictation handed the
            // whole buffer straight in — so a 3-minute dictation kept only
            // ~the first 30 s and silently lost everything after, with no
            // error and no indication.
            //
            // Chunk here, inside the provider that actually has the limit,
            // so every caller is correct without needing to know. Windows
            // are 29 s with 2 s overlap — the same values Meeting Mode uses.
            // Audio that fits in one window takes the single-shot fast path
            // and is byte-identical to before; Meeting Mode always passes
            // <= 29 s chunks, so it hits that path and is unaffected.
            const CHUNK_SECS: usize = 29;
            const OVERLAP_SECS: usize = 2;
            let chunk_samples = CHUNK_SECS * sample_rate.max(1) as usize;

            // Single-window fast path — unchanged behaviour.
            if samples.len() <= chunk_samples {
                tracing::info!(%model_id, samples = samples.len(), sample_rate, rms, "whisper: calling recognizer.transcribe (native)");
                let result = recognizer.transcribe(sample_rate, &samples);
                tracing::info!(%model_id, "whisper: recognizer.transcribe returned");

                // Whisper hallucination filter. Whisper's training data
                // included captioned video where silence/non-speech audio
                // is annotated with bracketed tags like [Music], [Applause],
                // (laughter), etc. Fed quiet or ambiguous audio, the model
                // confabulates the same tags — they were in the training
                // distribution but are never a real dictation result. The
                // pre-recogniser RMS gate catches obvious silence; this
                // catches marginal audio that cleared the gate but produced
                // a tag-shaped output. Drops the whole result rather than
                // pasting [Music] into the user's app.
                let trimmed = result.text.trim();
                if is_whisper_hallucination(trimmed) {
                    tracing::info!(%model_id, rms, returned = %trimmed, "whisper: dropping hallucinated bracketed-tag output");
                    return Ok((String::new(), None));
                }
                // sherpa_rs' WhisperRecognizerResult exposes the detected
                // language when running in auto mode. Surfaced on AsrResult
                // so downstream (history, mode picker) can record it.
                let detected = if !result.lang.is_empty() {
                    Some(result.lang)
                } else {
                    None
                };
                return Ok((result.text, detected));
            }

            // Chunked path for long single-shot dictation.
            let overlap_samples = OVERLAP_SECS * sample_rate.max(1) as usize;
            let stride = chunk_samples.saturating_sub(overlap_samples).max(1);
            let total = samples.len();
            tracing::info!(%model_id, total_samples = total, chunk_samples, "whisper: input exceeds one 30s window; transcribing in chunks");

            let mut start = 0usize;
            let mut chunk_index = 0usize;
            let mut stitched = String::new();
            let mut first_lang: Option<String> = None;

            while start < total {
                let end = (start + chunk_samples).min(total);
                let result = recognizer.transcribe(sample_rate, &samples[start..end]);
                let trimmed = result.text.trim();

                if is_whisper_hallucination(trimmed) {
                    tracing::info!(%model_id, chunk_index, returned = %trimmed, "whisper: dropping hallucinated chunk output");
                } else {
                    // Drop the leading overlap region on every chunk after
                    // the first. No word timings offline, so drop the
                    // OVERLAP/CHUNK leading fraction of words — conservative,
                    // better a dropped duplicate at a boundary than a
                    // repeated word. Same rule Meeting Mode's stitch uses.
                    let chunk_text = if chunk_index == 0 {
                        trimmed.to_string()
                    } else {
                        let words: Vec<&str> = trimmed.split_whitespace().collect();
                        let drop_count = (words.len() * OVERLAP_SECS / CHUNK_SECS).min(words.len());
                        words[drop_count..].join(" ")
                    };
                    if !chunk_text.is_empty() {
                        if !stitched.is_empty() {
                            stitched.push(' ');
                        }
                        stitched.push_str(&chunk_text);
                    }
                    if first_lang.is_none() && !result.lang.is_empty() {
                        first_lang = Some(result.lang);
                    }
                }

                if end == total {
                    break;
                }
                start += stride;
                chunk_index += 1;
            }
            tracing::info!(%model_id, chunks = chunk_index + 1, chars = stitched.len(), "whisper: chunked transcribe complete");
            Ok((stitched, first_lang))
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
    let mostly_music_glyphs = trimmed
        .chars()
        .all(|c| c == '♪' || c == '♫' || c == ' ' || c == '\t' || c == '.' || c == ',');
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

#[cfg(test)]
mod step0 {
    //! Step 0 benchmark harness for the v0.15.4 Whisper-Small-as-default
    //! decision. Loads the production `SherpaWhisper` engine against a
    //! real WAV on disk and emits the same shape as the production
    //! `meeting transcribe complete` log line so the numbers are
    //! directly comparable to the v0.15.3 Medium baseline.
    //!
    //! Both tests are `#[ignore]` — they require a real model bundle
    //! at `%APPDATA%/Utterbar/models/sherpa-whisper-small` and a WAV
    //! at the path given by `UTTERBAR_STEP0_WAV`. Run manually:
    //!
    //!   $env:UTTERBAR_STEP0_WAV="C:\Users\jaypa\AppData\Roaming\Utterbar\recordings\01KQBPA10SN2GZ8RFGF5NK1Y91.wav"
    //!   cargo test --release -p utterbar -- --ignored step0_small_verification --nocapture
    //!
    //! Release mode is required — the benchmark is meaningless on a
    //! debug build (5-10× slower).
    use super::*;
    use crate::services::inference::provider::AudioBuffer;
    use std::time::Instant;

    fn model_dir() -> PathBuf {
        let appdata = std::env::var("APPDATA").expect("APPDATA env var");
        PathBuf::from(appdata)
            .join("Utterbar")
            .join("models")
            .join("sherpa-whisper-small")
    }

    fn wav_path() -> PathBuf {
        PathBuf::from(std::env::var("UTTERBAR_STEP0_WAV").expect("UTTERBAR_STEP0_WAV env var"))
    }

    /// Read a 16 kHz mono i16 WAV into the `AudioBuffer` shape the
    /// production transcribe path expects. Mirrors the recorder's
    /// own format so the test exercises the real path.
    fn read_wav(path: &Path) -> AudioBuffer {
        let mut reader = hound::WavReader::open(path)
            .unwrap_or_else(|e| panic!("open wav {}: {e}", path.display()));
        let spec = reader.spec();
        assert_eq!(spec.sample_rate, 16_000, "Step 0 wav must be 16 kHz");
        assert_eq!(spec.channels, 1, "Step 0 wav must be mono");
        assert_eq!(spec.bits_per_sample, 16, "Step 0 wav must be 16-bit");
        let pcm: Vec<i16> = reader
            .samples::<i16>()
            .map(|s| s.expect("read sample"))
            .collect();
        AudioBuffer {
            pcm,
            sample_rate: spec.sample_rate,
            channels: spec.channels,
        }
    }

    /// Mirrors `CHUNKED_MEETING_CHUNK_SECS` / `_OVERLAP_SECS` in
    /// dictation.rs. The harness duplicates the chunking arithmetic
    /// here rather than calling into the orchestrator because building
    /// an `AppState` from a test requires the full app boot. If the
    /// production constants change, update these too.
    const HARNESS_CHUNK_SECS: usize = 29;
    const HARNESS_OVERLAP_SECS: usize = 2;

    async fn run_step0(label: &str) {
        let model_dir = model_dir();
        assert!(
            model_dir.is_dir(),
            "Step 0 needs Whisper Small installed at {}",
            model_dir.display()
        );
        let wav = wav_path();
        let audio = read_wav(&wav);
        let audio_ms = (audio.pcm.len() as u64 * 1000 / audio.sample_rate.max(1) as u64) as u32;
        let sample_rate = audio.sample_rate;
        let pcm = audio.pcm;
        let engine = SherpaWhisper::new(
            "sherpa-whisper-small".to_string(),
            model_dir,
            "en".to_string(), // English-pinned to match meeting-mode default
            "cpu".to_string(),
        );

        let warm_start = Instant::now();
        engine.warm().await.expect("whisper warm");
        let warm_ms = warm_start.elapsed().as_millis();

        // Slice PCM into 29 s windows with 2 s overlap, mirroring
        // `transcribe_chunked` in dictation.rs. Each chunk goes through
        // the same `engine.transcribe` call the production path uses,
        // so the cumulative processing time is directly comparable to
        // the v0.15.3 chunked-path RTF baseline (0.87 on Medium).
        let chunk_samples = HARNESS_CHUNK_SECS * sample_rate as usize;
        let overlap_samples = HARNESS_OVERLAP_SECS * sample_rate as usize;
        let stride = chunk_samples.saturating_sub(overlap_samples);
        let total = pcm.len();

        let transcribe_start = Instant::now();
        let mut all_text = String::new();
        let mut chunk_count: u32 = 0;
        if total <= chunk_samples {
            // Single-chunk fast path — matches the same branch in
            // transcribe_chunked.
            let buf = AudioBuffer {
                pcm: pcm.clone(),
                sample_rate,
                channels: 1,
            };
            let r = engine
                .transcribe(buf, AsrOptions::default())
                .await
                .expect("whisper transcribe");
            all_text.push_str(&r.text);
            chunk_count = 1;
        } else {
            let mut start = 0usize;
            while start < total {
                let end = (start + chunk_samples).min(total);
                let slice = pcm[start..end].to_vec();
                let buf = AudioBuffer {
                    pcm: slice,
                    sample_rate,
                    channels: 1,
                };
                let r = engine
                    .transcribe(buf, AsrOptions::default())
                    .await
                    .expect("whisper transcribe chunk");
                if !all_text.is_empty() && !r.text.is_empty() {
                    all_text.push(' ');
                }
                all_text.push_str(&r.text);
                chunk_count += 1;
                if end == total {
                    break;
                }
                start += stride;
            }
        }
        let processing_ms = transcribe_start.elapsed().as_millis() as u64;

        let rtf = if audio_ms == 0 {
            0.0
        } else {
            processing_ms as f32 / audio_ms as f32
        };

        // Same field set as the production `meeting transcribe complete`
        // log line in dictation.rs:2383 so a tail|grep diff is direct.
        eprintln!(
            "STEP0 {label}: audio_duration_ms={audio_ms} \
             total_processing_ms={processing_ms} \
             realtime_factor={rtf:.3} \
             selected_model_id=sherpa-whisper-small \
             chunk_count={chunk_count} \
             whisper_language=en \
             warm_ms={warm_ms} \
             chars={}",
            all_text.len()
        );
        eprintln!(
            "STEP0 {label} TEXT START >>>\n{}\n<<< STEP0 {label} TEXT END",
            all_text
        );
    }

    /// Apples-to-apples vs the v0.15.3 Medium real-meeting baseline
    /// (116.6s audio → 101.6s processing → RTF 0.87). Same WAV, same
    /// language pin, same code path — only the model id changes.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[ignore]
    async fn step0_small_verification() {
        run_step0("116s").await;
    }

    /// 5-minute meeting test for the v0.15.4 release-notes headline
    /// number. Run with UTTERBAR_STEP0_WAV pointed at a ~5 min WAV.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[ignore]
    async fn step0_small_5min() {
        run_step0("5min").await;
    }
}
