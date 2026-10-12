//! v0.15.6 DirectML smoke regression suite (was Step 2 of the
//! v0.15.6 development).
//!
//! Drives the production engine code paths (`SherpaWhisper`,
//! `SherpaParakeet`) against a real WAV and a real installed model
//! directory, with provider strings `"cpu"` and `"directml"`. The
//! intent is to keep this around as a manual-run regression suite
//! when the vendored sherpa-onnx + ORT + DirectML bundle gets
//! rebuilt — a future ORT/DML version bump that quietly breaks
//! Whisper or Parakeet should fail one of these cells in seconds.
//!
//! All tests are `#[ignore]` and gated on the env var
//! `UTTERBAR_STEP2_WAV` pointing at a 16 kHz mono 16-bit WAV. Run
//! individual cells from the command line so each test runs in
//! process isolation -- a native crash in one cell doesn't take the
//! sibling cells down.
//!
//! Example:
//!
//!   $env:UTTERBAR_STEP2_WAV="C:\Users\jaypa\AppData\Roaming\Utterbar\recordings\01KQBPA10SN2GZ8RFGF5NK1Y91.wav"
//!   cargo test --release -- --ignored step2_whisper_small_cpu --nocapture
//!
//! Output per cell:
//!   provider, model, warm_ms, audio_ms, transcribe_ms, rtf, chars,
//!   plus the transcribed text for eyeball comparison.
//!
//! See memory/v0156_directml_step2_results.md for the original Step 2
//! results and per-model speedup numbers.

use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::services::inference::provider::{AsrOptions, AsrProvider, AudioBuffer};
use crate::services::inference::sherpa_parakeet::SherpaParakeet;
use crate::services::inference::sherpa_whisper::SherpaWhisper;

fn appdata_models() -> PathBuf {
    let appdata = std::env::var("APPDATA").expect("APPDATA env var");
    PathBuf::from(appdata).join("Utterbar").join("models")
}

fn wav_path() -> PathBuf {
    PathBuf::from(std::env::var("UTTERBAR_STEP2_WAV").expect("UTTERBAR_STEP2_WAV env var"))
}

fn read_wav(path: &Path) -> AudioBuffer {
    let mut reader =
        hound::WavReader::open(path).unwrap_or_else(|e| panic!("open wav {}: {e}", path.display()));
    let spec = reader.spec();
    assert_eq!(spec.sample_rate, 16_000, "Step 2 wav must be 16 kHz");
    assert_eq!(spec.channels, 1, "Step 2 wav must be mono");
    assert_eq!(spec.bits_per_sample, 16, "Step 2 wav must be 16-bit");
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

/// Slice the audio to the first ~30 seconds. Step 2 is a smoke test —
/// we want quick passes per cell, not full meeting-mode RTF
/// measurements. 30 s also keeps us safely under sherpa-onnx's
/// 30-second offline-Whisper window so single-shot transcribe
/// works without chunking.
fn first_30s(audio: &AudioBuffer) -> AudioBuffer {
    let cap = 30 * audio.sample_rate as usize;
    let pcm = if audio.pcm.len() > cap {
        audio.pcm[..cap].to_vec()
    } else {
        audio.pcm.clone()
    };
    AudioBuffer {
        pcm,
        sample_rate: audio.sample_rate,
        channels: audio.channels,
    }
}

async fn smoke_whisper(model_dir_name: &str, model_id: &str, provider: &str) {
    let model_dir = appdata_models().join(model_dir_name);
    assert!(
        model_dir.is_dir(),
        "Step 2 needs {model_dir_name} installed at {}",
        model_dir.display()
    );
    let audio = first_30s(&read_wav(&wav_path()));
    let audio_ms = (audio.pcm.len() as u64 * 1000 / audio.sample_rate.max(1) as u64) as u32;
    let engine = SherpaWhisper::new(
        model_id.to_string(),
        model_dir,
        "en".to_string(),
        provider.to_string(),
    );

    let warm_start = Instant::now();
    let warm_result = engine.warm().await;
    let warm_ms = warm_start.elapsed().as_millis();
    let warm_ok = warm_result.is_ok();
    if let Err(e) = &warm_result {
        eprintln!("STEP2 {provider} {model_id}: WARM FAILED warm_ms={warm_ms} error={e:?}");
        return;
    }

    let t_start = Instant::now();
    let result = engine.transcribe(audio, AsrOptions::default()).await;
    let transcribe_ms = t_start.elapsed().as_millis() as u64;

    match result {
        Ok(r) => {
            let rtf = if audio_ms == 0 {
                0.0
            } else {
                transcribe_ms as f32 / audio_ms as f32
            };
            eprintln!(
                "STEP2 OK provider={provider} model={model_id} warm_ms={warm_ms} \
                 warm_ok={warm_ok} audio_ms={audio_ms} transcribe_ms={transcribe_ms} \
                 rtf={rtf:.3} chars={}",
                r.text.len()
            );
            eprintln!("STEP2 {provider} {model_id} TEXT >>>\n{}\n<<<", r.text);
        }
        Err(e) => {
            eprintln!(
                "STEP2 TRANSCRIBE FAILED provider={provider} model={model_id} warm_ms={warm_ms} \
                 transcribe_ms={transcribe_ms} error={e:?}"
            );
        }
    }
}

async fn smoke_parakeet(model_dir_name: &str, model_id: &str, provider: &str) {
    let model_dir = appdata_models().join(model_dir_name);
    assert!(
        model_dir.is_dir(),
        "Step 2 needs {model_dir_name} installed at {}",
        model_dir.display()
    );
    let audio = first_30s(&read_wav(&wav_path()));
    let audio_ms = (audio.pcm.len() as u64 * 1000 / audio.sample_rate.max(1) as u64) as u32;
    let engine = SherpaParakeet::new(model_id.to_string(), model_dir, provider.to_string());

    let warm_start = Instant::now();
    let warm_result = engine.warm().await;
    let warm_ms = warm_start.elapsed().as_millis();
    if let Err(e) = &warm_result {
        eprintln!("STEP2 {provider} {model_id}: WARM FAILED warm_ms={warm_ms} error={e:?}");
        return;
    }

    let t_start = Instant::now();
    let result = engine.transcribe(audio, AsrOptions::default()).await;
    let transcribe_ms = t_start.elapsed().as_millis() as u64;

    match result {
        Ok(r) => {
            let rtf = if audio_ms == 0 {
                0.0
            } else {
                transcribe_ms as f32 / audio_ms as f32
            };
            eprintln!(
                "STEP2 OK provider={provider} model={model_id} warm_ms={warm_ms} \
                 audio_ms={audio_ms} transcribe_ms={transcribe_ms} \
                 rtf={rtf:.3} chars={}",
                r.text.len()
            );
            eprintln!("STEP2 {provider} {model_id} TEXT >>>\n{}\n<<<", r.text);
        }
        Err(e) => {
            eprintln!(
                "STEP2 TRANSCRIBE FAILED provider={provider} model={model_id} \
                 warm_ms={warm_ms} transcribe_ms={transcribe_ms} error={e:?}"
            );
        }
    }
}

// ── Whisper Small ─────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore]
async fn step2_whisper_small_cpu() {
    smoke_whisper("sherpa-whisper-small", "sherpa-whisper-small", "cpu").await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore]
async fn step2_whisper_small_dml() {
    smoke_whisper("sherpa-whisper-small", "sherpa-whisper-small", "directml").await;
}

// ── Whisper Medium ────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore]
async fn step2_whisper_medium_cpu() {
    smoke_whisper("sherpa-whisper-medium", "sherpa-whisper-medium", "cpu").await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore]
async fn step2_whisper_medium_dml() {
    smoke_whisper("sherpa-whisper-medium", "sherpa-whisper-medium", "directml").await;
}

// ── Whisper Large v3 ──────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore]
async fn step2_whisper_large_cpu() {
    smoke_whisper("sherpa-whisper-large-v3", "sherpa-whisper-large-v3", "cpu").await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore]
async fn step2_whisper_large_dml() {
    smoke_whisper(
        "sherpa-whisper-large-v3",
        "sherpa-whisper-large-v3",
        "directml",
    )
    .await;
}

// ── Whisper Turbo ─────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore]
async fn step2_whisper_turbo_cpu() {
    smoke_whisper("sherpa-whisper-turbo", "sherpa-whisper-turbo", "cpu").await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore]
async fn step2_whisper_turbo_dml() {
    smoke_whisper("sherpa-whisper-turbo", "sherpa-whisper-turbo", "directml").await;
}

// ── Parakeet TDT v3 int8 ──────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore]
async fn step2_parakeet_int8_cpu() {
    smoke_parakeet(
        "sherpa-parakeet-tdt-0.6b-v3-int8",
        "sherpa-parakeet-tdt-0.6b-v3-int8",
        "cpu",
    )
    .await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore]
async fn step2_parakeet_int8_dml() {
    smoke_parakeet(
        "sherpa-parakeet-tdt-0.6b-v3-int8",
        "sherpa-parakeet-tdt-0.6b-v3-int8",
        "directml",
    )
    .await;
}
