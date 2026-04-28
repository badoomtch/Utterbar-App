//! cpal capture. The cpal `Stream` is `!Send` on Windows, so it is parked on a dedicated
//! OS thread; all cross-thread communication happens through channels. Downstream always
//! receives 16 kHz mono Int16.

use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat, StreamConfig};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::sync::mpsc;

use super::resample::StatefulResampler;
use super::InputDevice;

pub const TARGET_RATE: u32 = 16_000;

/// Safety cap on the in-thread PCM buffer. The cap exists as a leak backstop:
/// if `stop()` is somehow never called, the buffer must not grow unbounded.
/// It is NOT meant to be a feature limit on recording length.
///
/// Sized at 8 hours: 8 × 60 × 60 × 16_000 = 460,800,000 samples ≈ 920 MB at
/// i16 mono 16 kHz. Comfortably above any plausible legitimate use (multi-
/// hour conferences, lectures, meetings) without permitting a runaway leak
/// to consume all of system RAM. Pre-0.12.1 the cap was 30 minutes and
/// triggered SILENTLY — the capture callback would simply stop appending,
/// the user would record for an hour and lose the last 30 minutes with no
/// warning. From 0.12.1 the cap logs a single warning per session when
/// hit, and the cap itself is comfortable enough that hitting it almost
/// always indicates a leak rather than a long meeting.
const MAX_BUFFERED_SAMPLES: usize = (TARGET_RATE as usize) * 60 * 60 * 8;

/// Append `pcm` to `buf` if there is headroom under the safety cap. If
/// the cap is hit, log a warning ONCE per `cap_hit` flag (caller-owned
/// AtomicBool) so the log doesn't flood when the callback fires every
/// few ms after the cap. Returns true when the append happened.
///
/// Centralising this so the three SampleFormat branches (F32 / I16 / U16)
/// don't drift apart on the truncation logic.
fn append_capped(
    buf: &mut Vec<i16>,
    pcm: &[i16],
    cap_hit: &AtomicBool,
    label: &'static str,
) -> bool {
    if buf.len() + pcm.len() <= MAX_BUFFERED_SAMPLES {
        buf.extend_from_slice(pcm);
        true
    } else {
        // Top up to exactly the cap if we're partly over, then drop the
        // tail. Better than dropping the whole chunk: the user still
        // gets a continuous recording up to the limit.
        let room = MAX_BUFFERED_SAMPLES.saturating_sub(buf.len());
        if room > 0 {
            buf.extend_from_slice(&pcm[..room]);
        }
        // First time we trip the cap, complain loudly. After that, stay
        // quiet — the cpal callback fires every few ms.
        if !cap_hit.swap(true, Ordering::AcqRel) {
            tracing::warn!(
                source = label,
                cap_samples = MAX_BUFFERED_SAMPLES,
                cap_minutes = MAX_BUFFERED_SAMPLES / (TARGET_RATE as usize * 60),
                "capture: max-recording-length safety cap reached; new audio is being dropped. \
                 stop the recording and start a new one."
            );
        }
        false
    }
}

/// Handle returned to the rest of the app. All fields are `Send + Sync` — the cpal
/// stream itself is confined to the dedicated capture thread.
pub struct CaptureHandle {
    stop_flag: Arc<AtomicBool>,
    /// Streaming PCM frames for consumers that want live partials. Not read by the
    /// current non-streaming dictation path (which drains `buffer` on stop) but kept
    /// for the streaming ASR path we expect to wire up next.
    #[allow(dead_code)]
    pub frames_rx: tokio::sync::Mutex<mpsc::Receiver<Vec<i16>>>,
    pub meter_rx: tokio::sync::Mutex<mpsc::Receiver<(f32, f32)>>,
    buffer: Arc<Mutex<Vec<i16>>>,
}

#[derive(Clone)]
pub struct CaptureFinal {
    pub pcm: Vec<i16>,
    pub sample_rate: u32,
    pub duration_ms: u64,
}

pub fn list_input_devices() -> Result<Vec<InputDevice>> {
    let host = cpal::default_host();
    let default = host.default_input_device();
    let default_name = default.as_ref().and_then(|d| d.name().ok());
    let mut out = Vec::new();
    for d in host.input_devices()? {
        let name = d.name().unwrap_or_else(|_| "Unknown".into());
        let cfgs: Vec<u32> = d
            .supported_input_configs()
            .map(|cfgs| cfgs.map(|c| c.max_sample_rate().0).collect())
            .unwrap_or_default();
        out.push(InputDevice {
            id: name.clone(),
            name: name.clone(),
            is_default: default_name.as_deref() == Some(&name),
            supported_sample_rates: cfgs,
        });
    }
    Ok(out)
}

pub fn start(device_id: Option<&str>) -> Result<CaptureHandle> {
    let (frames_tx, frames_rx) = mpsc::channel::<Vec<i16>>(64);
    let (meter_tx, meter_rx) = mpsc::channel::<(f32, f32)>(64);
    let buffer = Arc::new(Mutex::new(Vec::<i16>::with_capacity(
        TARGET_RATE as usize * 60,
    )));
    let stop_flag = Arc::new(AtomicBool::new(false));

    // Confirm we can build the stream before returning, so the caller gets a sensible
    // error synchronously rather than a silent noop.
    let (ready_tx, ready_rx) = std::sync::mpsc::channel::<Result<(), String>>();

    let device_id_owned = device_id.map(|s| s.to_string());
    let buf_clone = buffer.clone();
    let stop_clone = stop_flag.clone();

    thread::Builder::new()
        .name("orliq-audio-capture".into())
        .spawn(move || {
            if let Err(e) = run_capture(
                device_id_owned.as_deref(),
                frames_tx,
                meter_tx,
                buf_clone,
                stop_clone,
                ready_tx,
            ) {
                tracing::error!(?e, "capture thread error");
            }
        })?;

    // Block briefly until the capture thread signals success or failure.
    match ready_rx.recv_timeout(std::time::Duration::from_millis(2_000)) {
        Ok(Ok(())) => {}
        Ok(Err(e)) => return Err(anyhow!(e)),
        Err(e) => return Err(anyhow!("capture thread did not signal ready: {e}")),
    }

    Ok(CaptureHandle {
        stop_flag,
        frames_rx: tokio::sync::Mutex::new(frames_rx),
        meter_rx: tokio::sync::Mutex::new(meter_rx),
        buffer,
    })
}

fn run_capture(
    device_id: Option<&str>,
    frames_tx: mpsc::Sender<Vec<i16>>,
    meter_tx: mpsc::Sender<(f32, f32)>,
    buffer: Arc<Mutex<Vec<i16>>>,
    stop_flag: Arc<AtomicBool>,
    ready_tx: std::sync::mpsc::Sender<Result<(), String>>,
) -> Result<()> {
    let host = cpal::default_host();
    let device = match device_id {
        Some(id) => host
            .input_devices()?
            .find(|d| d.name().map(|n| n == id).unwrap_or(false))
            .ok_or_else(|| anyhow!("input device not found: {id}"))?,
        None => host
            .default_input_device()
            .ok_or_else(|| anyhow!("no default input device"))?,
    };

    // Pick a config that respects the device's NATIVE sample rate
    // and a sample format we can decode.
    //
    // History:
    //   * v0.14.7 forced 48 kHz to fix tinny audio. That was the
    //     wrong fix — the real cause was the linear-interpolation
    //     resampler destroying high-rate audio on the way to 16 kHz.
    //     Forcing rate also landed on U8 configs that we couldn't
    //     decode (broke recording).
    //   * v0.14.8 filtered the rate-forced picker to F32/I16/U16
    //     formats. Recording worked again but was still fighting
    //     the device.
    //   * v0.14.9 (this) goes back to the device's default rate and
    //     picks the best decodable format within configs that cover
    //     it. The proper fix for tinny audio is the FFT-based
    //     resampler in `super::resample::StatefulResampler`, not
    //     fighting the driver. Same pattern as Handy
    //     (github.com/cjpais/Handy, MIT) which has soaked across
    //     thousands of installs without rate-related complaints.
    fn format_score(fmt: SampleFormat) -> u8 {
        // F32 highest because it requires no quantisation and is the
        // most common format for quality interfaces. I16 / U16
        // covered for completeness. Anything else returns 0 — only
        // selected if no decodable format exists at the device's
        // default rate, in which case the dispatch below errors out
        // cleanly.
        match fmt {
            SampleFormat::F32 => 4,
            SampleFormat::I16 => 3,
            SampleFormat::U16 => 2,
            _ => 0,
        }
    }

    let default_config = match device.default_input_config() {
        Ok(c) => c,
        Err(e) => {
            let _ = ready_tx.send(Err(format!("default_input_config: {e}")));
            return Err(anyhow!("default_input_config: {e}"));
        }
    };

    // Fast path: if the device's default config already gives us a
    // format we decode (F32/I16/U16), use it directly. Skips the
    // slow `supported_input_configs()` enumeration entirely on the
    // happy path, which on Windows queries WASAPI for every
    // (rate, format, channels) combo and adds 100-300 ms to the
    // hotkey-to-listening latency. Pre-v0.14.7 we did exactly this;
    // v0.14.7-9 enumerated unconditionally and the user noticed the
    // slowdown.
    //
    // Slow path: only when the default's format isn't decodable do
    // we enumerate, looking for a config at the same rate with a
    // format we can decode. Same shape as Handy
    // (github.com/cjpais/Handy, MIT). Most consumer mics never
    // enter this branch.
    let supported = if format_score(default_config.sample_format()) > 0 {
        default_config
    } else {
        let target_rate = default_config.sample_rate();
        match device.supported_input_configs() {
            Ok(configs) => {
                let mut best: Option<cpal::SupportedStreamConfigRange> = None;
                for range in configs {
                    if range.min_sample_rate() <= target_rate
                        && range.max_sample_rate() >= target_rate
                        && format_score(range.sample_format()) > 0
                    {
                        let pick = match best.as_ref() {
                            None => true,
                            Some(cur) => {
                                format_score(range.sample_format())
                                    > format_score(cur.sample_format())
                            }
                        };
                        if pick {
                            best = Some(range);
                        }
                    }
                }
                match best {
                    Some(range) => range.with_sample_rate(target_rate),
                    None => default_config,
                }
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "supported_input_configs unavailable, using device default config"
                );
                default_config
            }
        }
    };
    let sample_format = supported.sample_format();
    let config: StreamConfig = supported.config();
    let src_rate = config.sample_rate.0;
    let channels = config.channels as usize;
    tracing::info!(
        device_name = %device.name().unwrap_or_else(|_| "unknown".into()),
        src_rate,
        channels,
        ?sample_format,
        "audio capture stream opened"
    );

    let err_fn = |e| tracing::error!(?e, "cpal stream error");

    let mut meter_accum: f32 = 0.0;
    let mut meter_peak: f32 = 0.0;
    let mut meter_count: u32 = 0;
    let meter_window = (src_rate as f32 / 30.0) as u32;

    // Once-per-session cap-hit flag, shared across all three sample-format
    // callbacks so the warning fires exactly once if the cap is ever
    // reached. Wrapped in Arc because the cpal callback closure is
    // `move`-captured into a separate thread.
    let cap_hit = Arc::new(AtomicBool::new(false));

    // Closure factory that builds the per-callback body. Each format
    // branch instantiates its own `StatefulResampler` captured by
    // move into the cpal callback closure, so the resampler's input
    // ring persists across callbacks. (cpal callbacks fire every few
    // ms with variable-size chunks; the stateful resampler buffers
    // until it has a full FFT window before emitting samples.)
    //
    // Meter accumulation and the buffer-append path are unchanged
    // from pre-v0.14.9 — the only real difference is `resample_f32`
    // (linear interpolation, low audio quality) became
    // `StatefulResampler::push` (rubato FftFixedIn, transparent).
    let stream_result = match sample_format {
        SampleFormat::F32 => {
            let frames_tx = frames_tx.clone();
            let meter_tx = meter_tx.clone();
            let buffer = buffer.clone();
            let cap_hit = cap_hit.clone();
            let mut resampler = StatefulResampler::new(src_rate, TARGET_RATE);
            device.build_input_stream(
                &config,
                move |data: &[f32], _| {
                    let mono = downmix_f32(data, channels);
                    let resampled = resampler.push(&mono);
                    if resampled.is_empty() {
                        return;
                    }
                    let pcm = f32_to_i16(&resampled);
                    for &s in &pcm {
                        let f = s as f32 / i16::MAX as f32;
                        meter_accum += f * f;
                        meter_peak = meter_peak.max(f.abs());
                        meter_count += 1;
                        if meter_count >= meter_window {
                            let rms = (meter_accum / meter_count as f32).sqrt();
                            let _ = meter_tx.try_send((rms, meter_peak));
                            meter_accum = 0.0;
                            meter_peak = 0.0;
                            meter_count = 0;
                        }
                    }
                    if let Ok(mut buf) = buffer.lock() {
                        append_capped(&mut buf, &pcm, &cap_hit, "capture/f32");
                    }
                    let _ = frames_tx.try_send(pcm);
                },
                err_fn,
                None,
            )
        }
        SampleFormat::I16 => {
            let frames_tx = frames_tx.clone();
            let meter_tx = meter_tx.clone();
            let buffer = buffer.clone();
            let cap_hit = cap_hit.clone();
            let mut resampler = StatefulResampler::new(src_rate, TARGET_RATE);
            device.build_input_stream(
                &config,
                move |data: &[i16], _| {
                    let mono = downmix_i16_to_f32(data, channels);
                    let resampled = resampler.push(&mono);
                    if resampled.is_empty() {
                        return;
                    }
                    let pcm = f32_to_i16(&resampled);
                    for &s in &pcm {
                        let f = s as f32 / i16::MAX as f32;
                        meter_accum += f * f;
                        meter_peak = meter_peak.max(f.abs());
                        meter_count += 1;
                        if meter_count >= meter_window {
                            let rms = (meter_accum / meter_count as f32).sqrt();
                            let _ = meter_tx.try_send((rms, meter_peak));
                            meter_accum = 0.0;
                            meter_peak = 0.0;
                            meter_count = 0;
                        }
                    }
                    if let Ok(mut buf) = buffer.lock() {
                        append_capped(&mut buf, &pcm, &cap_hit, "capture/i16");
                    }
                    let _ = frames_tx.try_send(pcm);
                },
                err_fn,
                None,
            )
        }
        SampleFormat::U16 => {
            let frames_tx = frames_tx.clone();
            let buffer = buffer.clone();
            let cap_hit = cap_hit.clone();
            let mut resampler = StatefulResampler::new(src_rate, TARGET_RATE);
            device.build_input_stream(
                &config,
                move |data: &[u16], _| {
                    // u16 → i16 → f32 mono → resampler. Two casts not
                    // ideal but rare path; most quality devices come
                    // back as F32 or I16.
                    let conv: Vec<i16> = data.iter().map(|v| v.to_sample::<i16>()).collect();
                    let mono = downmix_i16_to_f32(&conv, channels);
                    let resampled = resampler.push(&mono);
                    if resampled.is_empty() {
                        return;
                    }
                    let pcm = f32_to_i16(&resampled);
                    if let Ok(mut buf) = buffer.lock() {
                        append_capped(&mut buf, &pcm, &cap_hit, "capture/u16");
                    }
                    let _ = frames_tx.try_send(pcm);
                },
                err_fn,
                None,
            )
        }
        other => {
            let msg = format!("unsupported sample format: {other:?}");
            let _ = ready_tx.send(Err(msg.clone()));
            return Err(anyhow!(msg));
        }
    };

    let stream = match stream_result {
        Ok(s) => s,
        Err(e) => {
            let _ = ready_tx.send(Err(format!("build_input_stream: {e}")));
            return Err(anyhow!("build_input_stream: {e}"));
        }
    };

    if let Err(e) = stream.play() {
        let _ = ready_tx.send(Err(format!("stream.play: {e}")));
        return Err(anyhow!("stream.play: {e}"));
    }
    let _ = ready_tx.send(Ok(()));

    // Park the thread until a stop is requested. The cpal stream lives in this scope,
    // so dropping it at the end of the loop correctly stops capture.
    while !stop_flag.load(Ordering::Relaxed) {
        thread::sleep(std::time::Duration::from_millis(50));
    }

    drop(stream);
    Ok(())
}

/// Safety net — if the outer service ever loses track of a `CaptureHandle` and drops it
/// without an explicit `stop()`, at least signal the capture thread to exit. Without this,
/// a forgotten handle would orphan the cpal stream and the capture thread forever.
impl Drop for CaptureHandle {
    fn drop(&mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
    }
}

impl CaptureHandle {
    /// Stop capture and hand back everything we buffered — including the tail.
    ///
    /// We used to just flip the stop flag, sleep 80 ms, and snapshot the buffer. That
    /// dropped roughly the last 100–300 ms of audio on most hardware, because:
    ///   1. WASAPI delivers data to cpal in ~10 ms chunks, but the kernel's own
    ///      capture buffer holds a few periods before the callback sees them. The
    ///      final word or syllable the user spoke is still in that buffer when they
    ///      hit stop.
    ///   2. There's always ~80–200 ms of mouth-closing / lip audio after the user
    ///      has "finished" speaking that they expect to be captured — pressing stop
    ///      is itself the end-of-word cue.
    ///   3. The old 80 ms sleep was shorter than the WASAPI flush latency on many
    ///      drivers, so callbacks that *would* have delivered the tail never got a
    ///      chance to fire before we snapshotted.
    ///
    /// New flow:
    ///   • Wait a fixed `TAIL_PAD_MS` first with the cpal stream STILL LIVE, so the
    ///     capture callback keeps appending to the buffer. This is the "grace tail"
    ///     — any audio the mic physically captured during this window still counts.
    ///   • Then flip the stop flag, which un-parks the capture thread. The thread
    ///     drops the stream, which flushes any in-flight WASAPI frames.
    ///   • Wait a short drain window for the drop to complete, THEN snapshot.
    ///   • Finally, check the buffer hasn't grown since the snapshot (rare, but can
    ///     happen on slow hosts) and re-snapshot if so.
    pub async fn stop(&self) -> CaptureFinal {
        // Grace tail — keep capturing for a beat past the user's stop gesture. 220 ms
        // covers typical WASAPI period + the user's end-of-word tail on the vast
        // majority of hardware without making the UI feel laggy.
        const TAIL_PAD_MS: u64 = 220;
        tokio::time::sleep(std::time::Duration::from_millis(TAIL_PAD_MS)).await;

        // Flip stop, then give the capture thread time to drop the cpal stream and
        // flush any last-callback frames. 120 ms is empirically enough on WASAPI;
        // low-latency drivers need ~20 ms, so this is generous.
        self.stop_flag.store(true, Ordering::Relaxed);
        tokio::time::sleep(std::time::Duration::from_millis(120)).await;

        // Snapshot — and re-check once in case a very slow host fired a final
        // callback between the first snapshot and now. Cheap insurance.
        let mut pcm = self.buffer.lock().unwrap().clone();
        tokio::time::sleep(std::time::Duration::from_millis(30)).await;
        let second = self.buffer.lock().unwrap().clone();
        if second.len() > pcm.len() {
            pcm = second;
        }

        let duration_ms = (pcm.len() as u64 * 1000) / TARGET_RATE as u64;
        CaptureFinal {
            pcm,
            sample_rate: TARGET_RATE,
            duration_ms,
        }
    }
}

// --- helpers ---

fn downmix_f32(data: &[f32], channels: usize) -> Vec<f32> {
    if channels == 1 {
        data.to_vec()
    } else {
        data.chunks_exact(channels)
            .map(|frame| frame.iter().sum::<f32>() / channels as f32)
            .collect()
    }
}

fn downmix_i16_to_f32(data: &[i16], channels: usize) -> Vec<f32> {
    if channels == 1 {
        data.iter().map(|s| *s as f32 / i16::MAX as f32).collect()
    } else {
        data.chunks_exact(channels)
            .map(|frame| {
                frame
                    .iter()
                    .map(|s| *s as f32 / i16::MAX as f32)
                    .sum::<f32>()
                    / channels as f32
            })
            .collect()
    }
}

/// Convert resampled f32 mono samples back to clamped i16 PCM. The
/// rest of the pipeline (history WAV writer, ASR providers) consumes
/// i16, so we serialise here at the boundary.
fn f32_to_i16(samples: &[f32]) -> Vec<i16> {
    samples
        .iter()
        .map(|s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
        .collect()
}
