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

/// What actually happened when we tried to honour the caller's
/// `device_id`. The dictation layer reads this so it can:
///   * log the resolved device alongside the requested one;
///   * raise a user-visible warning when the requested device wasn't
///     usable and we silently fell back to the host default — without
///     this, "the Focusrite isn't plugged in" looks identical to "the
///     Focusrite is plugged in but recording is broken" from the bar.
#[derive(Clone, Debug)]
pub struct CaptureMeta {
    /// Name cpal reported for the device we ultimately opened. This
    /// is the canonical "what mic actually recorded you" answer.
    pub resolved_device_name: String,
    /// `Some(reason)` when the caller asked for a specific device but
    /// we ended up on a different one. `None` on the happy path.
    pub fallback_reason: Option<DeviceFallbackReason>,
}

/// Why the requested device couldn't be honoured. Kept narrow — the
/// dictation layer maps each variant to user-visible copy, so the set
/// is closed and exhaustive.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeviceFallbackReason {
    /// The configured device id didn't match any device cpal currently
    /// knows about. Most common cause: USB mic unplugged, or the user
    /// renamed the device in Sound settings between sessions.
    NotFound { requested: String },
    /// cpal's `input_devices()` enumeration itself failed. Rare on
    /// Windows but can happen if the audio service is restarting.
    EnumerationFailed { error: String },
}

impl DeviceFallbackReason {
    /// Short human copy for the OS-level dictation toast. The format
    /// has to read on a Windows notification bubble in two lines —
    /// keep it terse and actionable.
    pub fn user_message(&self, resolved: &str) -> String {
        match self {
            Self::NotFound { requested } => format!(
                "Selected microphone \"{requested}\" isn't available — recording with \"{resolved}\" instead. Plug the device back in or pick another one in Audio settings."
            ),
            Self::EnumerationFailed { error } => format!(
                "Couldn't enumerate audio devices ({error}) — recording with \"{resolved}\". Try restarting Utterbar if this keeps happening."
            ),
        }
    }
}

/// Pure helper for matching a caller-supplied device id against a list
/// of `(id, name)` pairs from cpal. Extracted from `run_capture` so we
/// can unit-test the matching rules without standing up a real audio
/// host.
///
/// Rules, in order:
///   1. If `requested == None` → pick the host default (returned as
///      the special `Match::Default` variant — the caller still has to
///      ask cpal for the actual device handle).
///   2. If `requested == Some(id)` and exactly one device matches by
///      name → pick that one.
///   3. If multiple devices share the same name (Windows "Microphone
///      (USB Audio Device)" with multiple identical USB mics) → pick
///      the first match. cpal doesn't expose stable per-device
///      identifiers on Windows so name is the only key we have. We
///      log a warning so a future ambiguous-name report is diagnosable.
///   4. If `requested == Some(id)` and nothing matches → return
///      `Match::FallbackNotFound`. The caller decides whether to fall
///      back silently or warn — we don't make policy here.
#[derive(Debug, PartialEq, Eq)]
pub enum DeviceMatch {
    /// `requested` was `None`; caller should ask cpal for the host
    /// default device.
    Default,
    /// Found exactly the requested device. Carries the index into the
    /// `(id, name)` slice the caller passed in.
    ExactByName { index: usize },
    /// Found multiple devices sharing this name; we picked the first
    /// occurrence. `index` is that pick; `total_matching` is how many
    /// duplicates exist in case the caller wants to log the ambiguity.
    AmbiguousByName { index: usize, total_matching: usize },
    /// No device matches the requested id. Caller will typically fall
    /// back to the host default and surface a warning.
    FallbackNotFound,
}

pub fn match_input_device(requested: Option<&str>, devices: &[(String, String)]) -> DeviceMatch {
    let Some(req) = requested else {
        return DeviceMatch::Default;
    };
    let mut first_match: Option<usize> = None;
    let mut count = 0usize;
    for (i, (id, name)) in devices.iter().enumerate() {
        if id == req || name == req {
            count += 1;
            if first_match.is_none() {
                first_match = Some(i);
            }
        }
    }
    match (first_match, count) {
        (None, _) => DeviceMatch::FallbackNotFound,
        (Some(idx), 1) => DeviceMatch::ExactByName { index: idx },
        (Some(idx), n) => DeviceMatch::AmbiguousByName {
            index: idx,
            total_matching: n,
        },
    }
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

pub fn start(device_id: Option<&str>) -> Result<(CaptureHandle, CaptureMeta)> {
    let (frames_tx, frames_rx) = mpsc::channel::<Vec<i16>>(64);
    let (meter_tx, meter_rx) = mpsc::channel::<(f32, f32)>(64);
    let buffer = Arc::new(Mutex::new(Vec::<i16>::with_capacity(
        TARGET_RATE as usize * 60,
    )));
    let stop_flag = Arc::new(AtomicBool::new(false));

    // Confirm we can build the stream before returning, so the caller gets a sensible
    // error synchronously rather than a silent noop. The `Ok` payload carries the
    // resolved device + fallback reason (if any) so the dictation layer can log and
    // surface it.
    let (ready_tx, ready_rx) = std::sync::mpsc::channel::<Result<CaptureMeta, String>>();

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
    let meta = match ready_rx.recv_timeout(std::time::Duration::from_millis(2_000)) {
        Ok(Ok(meta)) => meta,
        Ok(Err(e)) => return Err(anyhow!(e)),
        Err(e) => return Err(anyhow!("capture thread did not signal ready: {e}")),
    };

    Ok((
        CaptureHandle {
            stop_flag,
            frames_rx: tokio::sync::Mutex::new(frames_rx),
            meter_rx: tokio::sync::Mutex::new(meter_rx),
            buffer,
        },
        meta,
    ))
}

fn run_capture(
    device_id: Option<&str>,
    frames_tx: mpsc::Sender<Vec<i16>>,
    meter_tx: mpsc::Sender<(f32, f32)>,
    buffer: Arc<Mutex<Vec<i16>>>,
    stop_flag: Arc<AtomicBool>,
    ready_tx: std::sync::mpsc::Sender<Result<CaptureMeta, String>>,
) -> Result<()> {
    let host = cpal::default_host();

    // Two-stage device resolution:
    //   1. Try to honour the caller's `device_id` against cpal's
    //      enumerated input devices.
    //   2. On any failure (id absent, list-enumeration error, name
    //      not present), fall back to the host default — but record
    //      WHY so the dictation layer can warn the user. Pre-fix, a
    //      missing configured device was silently swapped for the
    //      Windows default; a Focusrite user with the Windows default
    //      set to the laptop's Intel mic-array would record from the
    //      wrong device with no warning.
    let mut fallback_reason: Option<DeviceFallbackReason> = None;
    let device = if let Some(req) = device_id {
        match host.input_devices() {
            Ok(iter) => {
                let listed: Vec<(cpal::Device, String)> = iter
                    .filter_map(|d| {
                        let n = d.name().ok()?;
                        Some((d, n))
                    })
                    .collect();
                let pairs: Vec<(String, String)> =
                    listed.iter().map(|(_, n)| (n.clone(), n.clone())).collect();
                match match_input_device(Some(req), &pairs) {
                    DeviceMatch::ExactByName { index } => listed.into_iter().nth(index).unwrap().0,
                    DeviceMatch::AmbiguousByName {
                        index,
                        total_matching,
                    } => {
                        tracing::warn!(
                            requested_device_id = req,
                            total_matching,
                            "multiple input devices share the requested name; using the first match"
                        );
                        listed.into_iter().nth(index).unwrap().0
                    }
                    DeviceMatch::FallbackNotFound => {
                        fallback_reason = Some(DeviceFallbackReason::NotFound {
                            requested: req.to_string(),
                        });
                        host.default_input_device().ok_or_else(|| {
                            let msg = "no default input device";
                            let _ = ready_tx.send(Err(msg.into()));
                            anyhow!(msg)
                        })?
                    }
                    // `Default` only happens when `requested == None`, which is
                    // not this branch. Spelt out so a future refactor can't
                    // silently misroute it.
                    DeviceMatch::Default => unreachable!(
                        "match_input_device returned Default for a non-None requested id"
                    ),
                }
            }
            Err(e) => {
                fallback_reason = Some(DeviceFallbackReason::EnumerationFailed {
                    error: e.to_string(),
                });
                host.default_input_device().ok_or_else(|| {
                    let msg = "no default input device";
                    let _ = ready_tx.send(Err(msg.into()));
                    anyhow!(msg)
                })?
            }
        }
    } else {
        host.default_input_device().ok_or_else(|| {
            let msg = "no default input device";
            let _ = ready_tx.send(Err(msg.into()));
            anyhow!(msg)
        })?
    };

    let resolved_device_name = device.name().unwrap_or_else(|_| "unknown".into());
    if let Some(reason) = &fallback_reason {
        tracing::warn!(
            ?reason,
            resolved_device_name = %resolved_device_name,
            "audio capture: requested device unavailable, fell back to host default"
        );
    }

    // Stage timings for the start_recording critical path. Each block's
    // wall-clock cost is logged at the end so a future "feels slow"
    // report can be diagnosed from a log slice alone — no live tracing
    // required. Pre-v0.14.7 baseline on a healthy WASAPI mic was 7-20 ms
    // total; significant drift past that points at a regression in one
    // of the labelled stages.
    let cfg_t = std::time::Instant::now();
    let supported = match device.default_input_config() {
        Ok(s) => s,
        Err(e) => {
            let _ = ready_tx.send(Err(format!("default_input_config: {e}")));
            return Err(anyhow!("default_input_config: {e}"));
        }
    };
    let cfg_ms = cfg_t.elapsed().as_millis() as u64;
    let sample_format = supported.sample_format();
    let config: StreamConfig = supported.config();
    let src_rate = config.sample_rate.0;
    let channels = config.channels as usize;
    tracing::info!(
        device_name = %resolved_device_name,
        ?sample_format,
        src_rate,
        channels,
        cfg_ms,
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

    let build_t = std::time::Instant::now();
    let stream_result = match sample_format {
        SampleFormat::F32 => {
            let frames_tx = frames_tx.clone();
            let meter_tx = meter_tx.clone();
            let buffer = buffer.clone();
            let cap_hit = cap_hit.clone();
            device.build_input_stream(
                &config,
                move |data: &[f32], _| {
                    let pcm = downmix_and_resample_f32(data, channels, src_rate);
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
            device.build_input_stream(
                &config,
                move |data: &[i16], _| {
                    let pcm = downmix_and_resample_i16(data, channels, src_rate);
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
            device.build_input_stream(
                &config,
                move |data: &[u16], _| {
                    let conv: Vec<i16> = data.iter().map(|v| v.to_sample::<i16>()).collect();
                    let pcm = downmix_and_resample_i16(&conv, channels, src_rate);
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
    let build_ms = build_t.elapsed().as_millis() as u64;

    let play_t = std::time::Instant::now();
    if let Err(e) = stream.play() {
        let _ = ready_tx.send(Err(format!("stream.play: {e}")));
        return Err(anyhow!("stream.play: {e}"));
    }
    let play_ms = play_t.elapsed().as_millis() as u64;
    tracing::info!(build_ms, play_ms, "audio capture stream armed");
    let _ = ready_tx.send(Ok(CaptureMeta {
        resolved_device_name: resolved_device_name.clone(),
        fallback_reason,
    }));

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
    /// Copy buffered samples from `offset` to the current end (empty if
    /// `offset` is already at/past the end). Lets the live transcriber
    /// pull just the new audio each poll without disturbing the buffer —
    /// the full `mem::take` still happens once at `stop`.
    pub fn snapshot_from(&self, offset: usize) -> Vec<i16> {
        match self.buffer.lock() {
            Ok(b) if offset < b.len() => b[offset..].to_vec(),
            _ => Vec::new(),
        }
    }

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
    ///   • Wait the full drain window (drop latency + late-callback grace) BEFORE
    ///     touching the buffer, so by the time we extract there are no further
    ///     writers to race.
    ///   • `mem::take` the buffer in a single pass — replacing the shared `Vec<i16>`
    ///     with an empty one and moving the populated allocation into our return
    ///     value. Avoids the previous double-clone (up to 2× 920 MB on the 8-hour
    ///     safety cap, ~19 MB twice on a 5-minute dictation) at the cost of a
    ///     theoretical late frame that lands AFTER the take being held in a buffer
    ///     nothing ever drains — the handle drops immediately after this call so
    ///     that allocation is freed within microseconds anyway.
    pub async fn stop(&self) -> CaptureFinal {
        // Grace tail — keep capturing for a beat past the user's stop gesture.
        // 500 ms covers the WASAPI capture period, hotkey-release → stop-call
        // latency, and the typical trailing-syllable / mouth-closing audio a
        // user expects to be captured after they hit stop. Users reported the
        // previous 220 ms still dropping the last word on natural sentence
        // endings; 500 ms removes that without making the stop feel sluggish.
        const TAIL_PAD_MS: u64 = 500;
        tokio::time::sleep(std::time::Duration::from_millis(TAIL_PAD_MS)).await;

        // Flip stop, then give the capture thread time to drop the cpal stream and
        // flush any last-callback frames. 120 ms is empirically enough on WASAPI;
        // low-latency drivers need ~20 ms, so this is generous. The extra 30 ms
        // covers the slow-host case where a final callback fires just after the
        // stream drop returns; previously we re-snapshotted to catch this, now we
        // simply wait long enough that the late callback has already landed.
        self.stop_flag.store(true, Ordering::Relaxed);
        tokio::time::sleep(std::time::Duration::from_millis(120)).await;
        tokio::time::sleep(std::time::Duration::from_millis(30)).await;

        // Single-pass extraction — move the populated buffer out, leaving an empty
        // Vec behind. No clone of the captured PCM, regardless of length.
        let pcm = std::mem::take(&mut *self.buffer.lock().unwrap());

        let duration_ms = (pcm.len() as u64 * 1000) / TARGET_RATE as u64;
        CaptureFinal {
            pcm,
            sample_rate: TARGET_RATE,
            duration_ms,
        }
    }
}

// --- helpers ---

fn downmix_and_resample_f32(data: &[f32], channels: usize, src_rate: u32) -> Vec<i16> {
    let mono: Vec<f32> = if channels == 1 {
        data.to_vec()
    } else {
        data.chunks_exact(channels)
            .map(|frame| frame.iter().sum::<f32>() / channels as f32)
            .collect()
    };
    let resampled = resample_f32(&mono, src_rate, TARGET_RATE);
    resampled
        .into_iter()
        .map(|s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
        .collect()
}

fn downmix_and_resample_i16(data: &[i16], channels: usize, src_rate: u32) -> Vec<i16> {
    let mono: Vec<f32> = if channels == 1 {
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
    };
    let resampled = resample_f32(&mono, src_rate, TARGET_RATE);
    resampled
        .into_iter()
        .map(|s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
        .collect()
}

fn resample_f32(input: &[f32], src: u32, dst: u32) -> Vec<f32> {
    if src == dst || input.is_empty() {
        return input.to_vec();
    }
    let ratio = src as f64 / dst as f64;
    let out_len = (input.len() as f64 / ratio).round() as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let pos = i as f64 * ratio;
        let p = pos.floor() as usize;
        let frac = (pos - p as f64) as f32;
        let a = *input.get(p).unwrap_or(&0.0);
        let b = *input.get(p + 1).unwrap_or(&a);
        out.push(a + (b - a) * frac);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 48 kHz → 16 kHz resampling preserves the duration of a clip
    /// within a single sample of round-off. v0.14.9's chunked FFT
    /// resampler dropped up to one full source-chunk (~1024 samples
    /// at 48 kHz, ~21 ms) at end-of-recording because partial windows
    /// were never flushed; this test pins the linear-interpolator
    /// behaviour so any future swap that re-introduces tail-loss
    /// fails CI.
    #[test]
    fn resample_48k_to_16k_preserves_duration() {
        let one_second_at_48k: Vec<f32> = (0..48_000)
            .map(|i| (i as f32 / 48_000.0 * 2.0 * std::f32::consts::PI * 440.0).sin())
            .collect();
        let out = resample_f32(&one_second_at_48k, 48_000, 16_000);
        assert!(
            (out.len() as i64 - 16_000).abs() <= 1,
            "expected ~16000 output samples, got {}",
            out.len()
        );
    }

    /// 16 kHz → 16 kHz is a true pass-through. No interpolation, no
    /// length drift, no amplitude attenuation.
    #[test]
    fn resample_passthrough_when_rates_match() {
        let input = vec![0.5_f32; 32_000];
        let out = resample_f32(&input, 16_000, 16_000);
        assert_eq!(out.len(), input.len());
        assert!(out.iter().all(|s| (*s - 0.5).abs() < 1e-6));
    }

    /// Resampling a steady-amplitude tone at speech levels must not
    /// collapse the amplitude — a regression where the resampler
    /// silently zeroed audio would show up as a near-silent peak.
    /// 0.3 is comfortable speech RMS in normalised f32 space.
    #[test]
    fn resample_does_not_collapse_amplitude() {
        let input: Vec<f32> = (0..48_000)
            .map(|i| (i as f32 / 48_000.0 * 2.0 * std::f32::consts::PI * 220.0).sin() * 0.3)
            .collect();
        let out = resample_f32(&input, 48_000, 16_000);
        let peak = out.iter().fold(0.0_f32, |a, b| a.max(b.abs()));
        assert!(
            peak > 0.2,
            "resampled peak collapsed to {peak} (expected > 0.2)"
        );
    }

    fn pairs(devs: &[&str]) -> Vec<(String, String)> {
        devs.iter()
            .map(|n| (n.to_string(), n.to_string()))
            .collect()
    }

    /// `requested == None` always resolves to `Default`, even when the
    /// device list is non-empty. This is the "user has no preference,
    /// follow the host default" path the floating bar hits today.
    #[test]
    fn match_none_means_default() {
        let devs = pairs(&["Mic A", "Mic B"]);
        assert_eq!(match_input_device(None, &devs), DeviceMatch::Default);
    }

    /// The exact-match path is the v0.14.6+ regression target — Jay's
    /// Focusrite was selected in settings but the bar invoked
    /// start_recording with `None`, so this test pins down that the
    /// matcher routes a real selection to the right device.
    #[test]
    fn match_finds_unique_name() {
        let devs = pairs(&["Mic A", "Focusrite", "Mic B"]);
        assert_eq!(
            match_input_device(Some("Focusrite"), &devs),
            DeviceMatch::ExactByName { index: 1 }
        );
    }

    /// Windows happily produces two identical "Microphone (USB Audio
    /// Device)" entries when the user has two of the same USB mic.
    /// cpal exposes no further disambiguator on Windows, so the
    /// matcher must pick the first occurrence and report the
    /// ambiguity for diagnostics.
    #[test]
    fn match_duplicate_names_picks_first_and_reports_count() {
        let devs = pairs(&["USB Mic", "USB Mic", "USB Mic"]);
        assert_eq!(
            match_input_device(Some("USB Mic"), &devs),
            DeviceMatch::AmbiguousByName {
                index: 0,
                total_matching: 3,
            }
        );
    }

    /// Missing-device path: caller asked for "Focusrite" but only the
    /// onboard mic is present. The matcher returns FallbackNotFound
    /// — the policy decision (silent fallback vs warn) is in the
    /// caller, not here.
    #[test]
    fn match_returns_fallback_when_no_device_matches() {
        let devs = pairs(&["Microphone Array (Intel Smart Sound)"]);
        assert_eq!(
            match_input_device(Some("Focusrite"), &devs),
            DeviceMatch::FallbackNotFound
        );
    }

    /// Empty device list: every requested id falls back. The host's
    /// own default lookup will then either succeed or fail explicitly.
    #[test]
    fn match_empty_device_list_falls_back() {
        let devs: Vec<(String, String)> = vec![];
        assert_eq!(
            match_input_device(Some("anything"), &devs),
            DeviceMatch::FallbackNotFound
        );
        assert_eq!(match_input_device(None, &devs), DeviceMatch::Default);
    }
}
