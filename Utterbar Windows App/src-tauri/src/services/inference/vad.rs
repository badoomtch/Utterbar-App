//! Silero VAD segmentation for Meeting Mode.
//!
//! Meeting Mode captures the microphone and the system loopback as two
//! separate streams and never mixes them for ASR. Each stream is run
//! through this VAD to extract speech-only spans, so the recogniser
//! never sees the long silent stretches where the *other* side is
//! talking. Those silent/near-silent stretches are exactly the
//! non-speech input that made the v0.15.0 meeting path hallucinate
//! foreign-language "ghost words"; gating on speech is how the wider
//! open-source ecosystem (screenpipe et al.) avoids it.
//!
//! Each speech span also carries its start sample, which is what lets
//! the two channels be merged back together by timestamp without ever
//! sample-aligning the two independent capture clocks.
//!
//! Backed by the Silero VAD compiled into the vendored sherpa-onnx
//! bundle (`sherpa_rs::silero_vad`), driven by the ~629 KB
//! `silero_vad.onnx` model bundled next to the executable.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use sherpa_rs::silero_vad::{SileroVad, SileroVadConfig};

/// Sample rate the whole audio pipeline is normalised to (16 kHz mono).
const VAD_SAMPLE_RATE: u32 = 16_000;

/// Silero's fixed analysis window. The model expects 512-sample
/// (32 ms @ 16 kHz) frames; feeding any other size is undefined.
const VAD_WINDOW: usize = 512;

/// Cap on a single speech span. Kept below the ~29 s Whisper decode
/// ceiling so every span is a single ASR call — a talker who never
/// pauses still gets cut here (and stitched back by the merge step)
/// rather than silently truncated inside sherpa's offline Whisper,
/// which drops everything past ~30 s.
///
/// NOTE: sherpa-rs's `SileroVadConfig::default()` ships
/// `max_speech_duration: 0.5`, which would chop every turn into
/// half-second slivers. We MUST set this explicitly.
const MAX_SPEECH_SECS: f32 = 25.0;

/// Ignore blips shorter than this — a click, a single "mm", a cough.
const MIN_SPEECH_SECS: f32 = 0.25;

/// A silence gap shorter than this doesn't end a turn; it keeps the
/// span going so a natural within-sentence pause doesn't shatter one
/// sentence into many fragments.
const MIN_SILENCE_SECS: f32 = 0.5;

/// One detected speech span within a single channel.
#[derive(Debug, Clone, PartialEq)]
pub struct SpeechSpan {
    /// Start offset of the span within the source buffer, in samples
    /// at 16 kHz. `start_sample / 16000` is the wall-clock second.
    pub start_sample: usize,
    /// The speech audio for this span — 16 kHz mono i16, ready to hand
    /// straight to the ASR provider.
    pub pcm: Vec<i16>,
}

impl SpeechSpan {
    /// Wall-clock start of this span, in whole seconds from the start
    /// of its channel's recording.
    pub fn start_secs(&self) -> u32 {
        (self.start_sample / VAD_SAMPLE_RATE as usize) as u32
    }
}

/// Resolve the bundled `silero_vad.onnx`.
///
/// In a packaged build it is bundled next to the executable via
/// `tauri.conf.json` → `bundle.resources`, so it sits in the directory
/// the ONNX runtime DLLs load from. Falls back to the committed
/// `src-tauri/vendor/silero_vad.onnx` so `cargo run`, `cargo test`, and
/// `tauri dev` resolve it without a packaged bundle.
pub fn resolve_model_path() -> Result<PathBuf> {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let p = dir.join("silero_vad.onnx");
            if p.is_file() {
                return Ok(p);
            }
        }
    }
    let dev = Path::new(env!("CARGO_MANIFEST_DIR")).join("vendor/silero_vad.onnx");
    if dev.is_file() {
        return Ok(dev);
    }
    Err(anyhow!(
        "silero_vad.onnx not found next to the executable or in vendor/"
    ))
}

/// Drain every completed speech segment the VAD currently holds into
/// `spans`, converting each back to 16 kHz i16.
fn drain_segments(vad: &mut SileroVad, spans: &mut Vec<SpeechSpan>) {
    while !vad.is_empty() {
        let seg = vad.front();
        let start_sample = seg.start.max(0) as usize;
        let pcm: Vec<i16> = seg
            .samples
            .iter()
            .map(|s| (s.clamp(-1.0, 1.0) * 32767.0) as i16)
            .collect();
        if !pcm.is_empty() {
            spans.push(SpeechSpan { start_sample, pcm });
        }
        vad.pop();
    }
}

/// Segment a 16 kHz mono i16 buffer into speech-only spans, in source
/// order.
///
/// Blocking and CPU-bound (ONNX inference over every 512-sample frame);
/// call it from `spawn_blocking`, never on an async worker.
pub fn segment_speech(pcm: &[i16], model_path: &Path) -> Result<Vec<SpeechSpan>> {
    if pcm.len() < VAD_WINDOW {
        return Ok(Vec::new());
    }
    let model = model_path
        .to_str()
        .ok_or_else(|| anyhow!("VAD model path is not valid UTF-8"))?;

    let config = SileroVadConfig {
        model: model.to_string(),
        min_silence_duration: MIN_SILENCE_SECS,
        min_speech_duration: MIN_SPEECH_SECS,
        max_speech_duration: MAX_SPEECH_SECS,
        threshold: 0.5,
        sample_rate: VAD_SAMPLE_RATE,
        window_size: VAD_WINDOW as i32,
        // The VAD model is tiny; CPU is the right (and only sensible)
        // provider. Pin it so a DirectML-capable build doesn't try to
        // route a 629 KB model through the GPU.
        provider: Some("cpu".to_string()),
        num_threads: Some(1),
        debug: false,
    };

    // `buffer_size_in_seconds` is the detector's internal ring; 30 s is
    // ample for our frame-at-a-time feed loop.
    let mut vad = SileroVad::new(config, 30.0).map_err(|e| anyhow!("create Silero VAD: {e}"))?;

    // The VAD consumes f32 windows in [-1, 1]; convert once up front.
    let samples: Vec<f32> = pcm.iter().map(|s| *s as f32 / 32768.0).collect();

    let mut spans = Vec::new();
    let mut i = 0;
    while i + VAD_WINDOW <= samples.len() {
        vad.accept_waveform(samples[i..i + VAD_WINDOW].to_vec());
        drain_segments(&mut vad, &mut spans);
        i += VAD_WINDOW;
    }
    // Flush the trailing partial window so a final utterance that
    // isn't followed by min_silence still surfaces.
    vad.flush();
    drain_segments(&mut vad, &mut spans);

    Ok(spans)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_model_path_finds_the_committed_copy() {
        // In a dev/test build there's no packaged bundle, so this
        // exercises the committed `vendor/silero_vad.onnx` fallback.
        // That file must be present (and un-gitignored) for Meeting
        // Mode to work at all.
        let p = resolve_model_path().expect("silero_vad.onnx must resolve");
        assert!(p.is_file(), "resolved VAD model path must be a file: {p:?}");
    }

    #[test]
    fn empty_or_tiny_buffer_yields_no_spans() {
        let model = resolve_model_path().unwrap();
        assert!(segment_speech(&[], &model).unwrap().is_empty());
        // Shorter than one analysis window: nothing to segment.
        assert!(segment_speech(&[0i16; 100], &model).unwrap().is_empty());
    }

    #[test]
    fn start_secs_maps_samples_to_wall_clock() {
        let span = SpeechSpan {
            start_sample: 16_000 * 12 + 500,
            pcm: vec![0; 4],
        };
        assert_eq!(span.start_secs(), 12);
    }

    /// End-to-end VAD load + run, plus real-speech detection.
    ///
    /// Two parts:
    ///   1. Always: segment a 1 s silence / 1 s 220 Hz tone / 1 s
    ///      silence buffer. This proves the FFI links, the model loads,
    ///      and inference runs without crashing. Silero is a *speech*
    ///      detector, so a pure tone yields few or zero spans — that
    ///      non-detection is the anti-ghost-word property we rely on, so
    ///      we assert only that it runs, not that it fires.
    ///   2. If `UTTERBAR_VAD_TEST_WAV` points at a 16 kHz mono WAV, read
    ///      it and assert real speech yields at least one span. This is
    ///      the positive-detection proof, kept out of bare CI (no asset)
    ///      but runnable manually.
    ///
    /// `#[ignore]` because it dynamically loads `sherpa-onnx-c-api.dll`,
    /// which must be on PATH at test runtime — true in a packaged build
    /// and when PATH includes `runtime/`, but not on a bare `cargo test`.
    /// Run with (from src-tauri):
    ///   PATH="$PWD/runtime:$PATH" UTTERBAR_VAD_TEST_WAV=/path/speech.wav \
    ///     cargo test --bin utterbar vad::tests -- --ignored --nocapture
    #[test]
    #[ignore]
    fn loads_model_and_detects_speech() {
        let model = resolve_model_path().unwrap();

        // (1) Load + run proof on a non-speech tone.
        let sr = VAD_SAMPLE_RATE as usize;
        let mut tone: Vec<i16> = vec![0i16; sr];
        for n in 0..sr {
            let t = n as f32 / sr as f32;
            let s = (2.0 * std::f32::consts::PI * 220.0 * t).sin() * 0.4;
            tone.push((s * 32767.0) as i16);
        }
        tone.resize(tone.len() + sr, 0);
        let _ = segment_speech(&tone, &model).expect("VAD must load and run without error");

        // (2) Positive detection on real speech, when an asset is given.
        if let Ok(wav_path) = std::env::var("UTTERBAR_VAD_TEST_WAV") {
            let mut reader = hound::WavReader::open(&wav_path).expect("open test WAV");
            assert_eq!(reader.spec().sample_rate, 16_000, "test WAV must be 16 kHz");
            let pcm: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap()).collect();
            let spans = segment_speech(&pcm, &model).expect("segment speech WAV");
            assert!(!spans.is_empty(), "real speech must yield >= 1 speech span");
            let total: usize = spans.iter().map(|s| s.pcm.len()).sum();
            eprintln!(
                "VAD: {} spans, {:.1}s speech, first span starts at {}s",
                spans.len(),
                total as f32 / VAD_SAMPLE_RATE as f32,
                spans[0].start_secs()
            );
        }
    }
}
