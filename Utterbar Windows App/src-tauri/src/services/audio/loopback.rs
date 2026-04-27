//! WASAPI loopback capture — records whatever is playing through the default render
//! endpoint (speakers / headphones / the default output device). Used when the user sets
//! Audio → Capture source to "System audio" or "Mixed".
//!
//! cpal does not support loopback on Windows as of 0.15, so we go direct to the Windows
//! API: `IMMDeviceEnumerator::GetDefaultAudioEndpoint(eRender)` followed by
//! `IAudioClient::Initialize(AUDCLNT_STREAMFLAGS_LOOPBACK)`. The resulting capture
//! client hands us the exact byte stream Windows is sending to the speakers — same
//! sample rate, same format as whatever the output device is running in shared mode.
//!
//! We downmix to mono f32 and hand the caller a channel of 16 kHz Int16 frames so the
//! rest of the pipeline (ASR, meter, preprocess) doesn't care where the audio came from.

#![cfg(windows)]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use anyhow::{anyhow, Result};
use tokio::sync::mpsc;
use windows::Win32::Foundation::WAIT_OBJECT_0;
use windows::Win32::Media::Audio::{
    eConsole, eRender, IAudioCaptureClient, IAudioClient, IMMDevice, IMMDeviceEnumerator,
    MMDeviceEnumerator, AUDCLNT_BUFFERFLAGS_SILENT, AUDCLNT_SHAREMODE_SHARED,
    AUDCLNT_STREAMFLAGS_EVENTCALLBACK, AUDCLNT_STREAMFLAGS_LOOPBACK, WAVEFORMATEX,
    WAVEFORMATEXTENSIBLE,
};
// These three wFormatTag constants don't live in the `windows` crate's `Audio` or
// `KernelStreaming` modules in 0.58; just define them locally — they're part of the
// Windows ABI and haven't changed since Vista.
const WAVE_FORMAT_PCM_TAG: u16 = 0x0001;
const WAVE_FORMAT_IEEE_FLOAT_TAG: u16 = 0x0003;
const WAVE_FORMAT_EXTENSIBLE_TAG: u16 = 0xFFFE;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_MULTITHREADED,
};
use windows::Win32::System::Threading::{CreateEventW, WaitForSingleObject};

use super::capture::TARGET_RATE;

/// Mirror of `capture::CaptureHandle` but powered by a loopback source. Exposes the same
/// channel shape so callers can swap sources without touching the downstream pipeline.
pub struct LoopbackHandle {
    stop_flag: Arc<AtomicBool>,
    /// Streaming PCM frames. Mirrors `CaptureHandle::frames_rx`; same rationale —
    /// kept for the streaming-ASR path, not read by the current stop-and-drain flow.
    #[allow(dead_code)]
    pub frames_rx: tokio::sync::Mutex<mpsc::Receiver<Vec<i16>>>,
    /// Live meter samples for the loopback source. Meter UI currently reads the
    /// primary mic path only; kept so a future "hear me + hear them" meter can
    /// consume the secondary stream without changing the handle shape.
    #[allow(dead_code)]
    pub meter_rx: tokio::sync::Mutex<mpsc::Receiver<(f32, f32)>>,
    buffer: Arc<Mutex<Vec<i16>>>,
}

/// Same safety net as `CaptureHandle::Drop` — if the handle gets dropped without an
/// explicit stop, terminate the WASAPI loopback thread so it doesn't orphan the audio
/// client and the in-thread PCM buffer.
impl Drop for LoopbackHandle {
    fn drop(&mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
    }
}

impl LoopbackHandle {
    pub async fn stop(&self) -> super::capture::CaptureFinal {
        self.stop_flag.store(true, Ordering::Relaxed);
        tokio::time::sleep(Duration::from_millis(100)).await;
        let pcm = self.buffer.lock().unwrap().clone();
        let duration_ms = (pcm.len() as u64 * 1000) / TARGET_RATE as u64;
        super::capture::CaptureFinal {
            pcm,
            sample_rate: TARGET_RATE,
            duration_ms,
        }
    }
}

pub fn start() -> Result<LoopbackHandle> {
    let (frames_tx, frames_rx) = mpsc::channel::<Vec<i16>>(64);
    let (meter_tx, meter_rx) = mpsc::channel::<(f32, f32)>(64);
    let buffer = Arc::new(Mutex::new(Vec::<i16>::with_capacity(
        TARGET_RATE as usize * 60,
    )));
    let stop_flag = Arc::new(AtomicBool::new(false));

    let (ready_tx, ready_rx) = std::sync::mpsc::channel::<Result<(), String>>();
    let buf_clone = buffer.clone();
    let stop_clone = stop_flag.clone();

    thread::Builder::new()
        .name("orliq-loopback-capture".into())
        .spawn(move || {
            if let Err(e) = run_loopback(frames_tx, meter_tx, buf_clone, stop_clone, ready_tx) {
                tracing::error!(?e, "loopback capture error");
            }
        })?;

    match ready_rx.recv_timeout(Duration::from_millis(3_000)) {
        Ok(Ok(())) => {}
        Ok(Err(e)) => return Err(anyhow!(e)),
        Err(e) => return Err(anyhow!("loopback thread did not signal ready: {e}")),
    }

    Ok(LoopbackHandle {
        stop_flag,
        frames_rx: tokio::sync::Mutex::new(frames_rx),
        meter_rx: tokio::sync::Mutex::new(meter_rx),
        buffer,
    })
}

fn run_loopback(
    frames_tx: mpsc::Sender<Vec<i16>>,
    meter_tx: mpsc::Sender<(f32, f32)>,
    buffer: Arc<Mutex<Vec<i16>>>,
    stop_flag: Arc<AtomicBool>,
    ready_tx: std::sync::mpsc::Sender<Result<(), String>>,
) -> Result<()> {
    unsafe {
        // COM init on this dedicated thread — MTA so we don't have to pump messages. The
        // return value from CoInitializeEx is HRESULT-ish; S_FALSE means "already init'd",
        // which is fine; we only bail on hard errors.
        let hr = CoInitializeEx(None, COINIT_MULTITHREADED);
        if hr.is_err() {
            let msg = format!("CoInitializeEx: {hr:?}");
            let _ = ready_tx.send(Err(msg.clone()));
            return Err(anyhow!(msg));
        }

        // Guard to ensure CoUninitialize runs even on early return.
        struct ComGuard;
        impl Drop for ComGuard {
            fn drop(&mut self) {
                unsafe { CoUninitialize() };
            }
        }
        let _com = ComGuard;

        let enumerator: IMMDeviceEnumerator =
            match CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) {
                Ok(e) => e,
                Err(e) => {
                    let msg = format!("CoCreateInstance(MMDeviceEnumerator): {e}");
                    let _ = ready_tx.send(Err(msg.clone()));
                    return Err(anyhow!(msg));
                }
            };

        let device: IMMDevice = match enumerator.GetDefaultAudioEndpoint(eRender, eConsole) {
            Ok(d) => d,
            Err(e) => {
                let msg = format!("GetDefaultAudioEndpoint(eRender): {e}");
                let _ = ready_tx.send(Err(msg.clone()));
                return Err(anyhow!(msg));
            }
        };

        let client: IAudioClient = match device.Activate(CLSCTX_ALL, None) {
            Ok(c) => c,
            Err(e) => {
                let msg = format!("IMMDevice::Activate(IAudioClient): {e}");
                let _ = ready_tx.send(Err(msg.clone()));
                return Err(anyhow!(msg));
            }
        };

        let mix_format_ptr = match client.GetMixFormat() {
            Ok(p) => p,
            Err(e) => {
                let msg = format!("GetMixFormat: {e}");
                let _ = ready_tx.send(Err(msg.clone()));
                return Err(anyhow!(msg));
            }
        };
        let wf: &WAVEFORMATEX = &*mix_format_ptr;
        let channels = wf.nChannels as usize;
        let src_rate = wf.nSamplesPerSec;
        let bits_per_sample = wf.wBitsPerSample;
        let format_tag = wf.wFormatTag as u32;
        // EXTENSIBLE format hides the "real" format tag in the SubFormat GUID, so assume
        // float if the mix is extensible + 32-bit (the overwhelmingly common case). Cheap
        // to check; wrong assumption only breaks for niche mixers running int32 loopback.
        let is_float = match format_tag {
            t if t == WAVE_FORMAT_IEEE_FLOAT_TAG as u32 => true,
            t if t == WAVE_FORMAT_PCM_TAG as u32 => false,
            // WAVE_FORMAT_EXTENSIBLE — peek at the SubFormat GUID.
            t if t == WAVE_FORMAT_EXTENSIBLE_TAG as u32 => {
                let ext: &WAVEFORMATEXTENSIBLE = &*(mix_format_ptr as *const WAVEFORMATEXTENSIBLE);
                ext.SubFormat.data1 == WAVE_FORMAT_IEEE_FLOAT_TAG as u32
            }
            _ => bits_per_sample == 32,
        };

        // Event-driven capture so we don't have to poll; Windows signals the event each
        // time a buffer is ready. Buffer duration of 100 ms is a balance between low
        // latency and not drowning the thread in tiny callbacks.
        let buf_duration_100ns: i64 = 100 * 10_000;

        let event = match CreateEventW(None, false, false, None) {
            Ok(h) => h,
            Err(e) => {
                let msg = format!("CreateEventW: {e}");
                let _ = ready_tx.send(Err(msg.clone()));
                return Err(anyhow!(msg));
            }
        };

        if let Err(e) = client.Initialize(
            AUDCLNT_SHAREMODE_SHARED,
            AUDCLNT_STREAMFLAGS_LOOPBACK | AUDCLNT_STREAMFLAGS_EVENTCALLBACK,
            buf_duration_100ns,
            0,
            mix_format_ptr,
            None,
        ) {
            let msg = format!("IAudioClient::Initialize: {e}");
            let _ = ready_tx.send(Err(msg.clone()));
            return Err(anyhow!(msg));
        }

        if let Err(e) = client.SetEventHandle(event) {
            let msg = format!("SetEventHandle: {e}");
            let _ = ready_tx.send(Err(msg.clone()));
            return Err(anyhow!(msg));
        }

        let capture_client: IAudioCaptureClient = match client.GetService() {
            Ok(c) => c,
            Err(e) => {
                let msg = format!("GetService(IAudioCaptureClient): {e}");
                let _ = ready_tx.send(Err(msg.clone()));
                return Err(anyhow!(msg));
            }
        };

        if let Err(e) = client.Start() {
            let msg = format!("IAudioClient::Start: {e}");
            let _ = ready_tx.send(Err(msg.clone()));
            return Err(anyhow!(msg));
        }

        let _ = ready_tx.send(Ok(()));

        // Meter accumulator — same shape as the mic capture path.
        let mut meter_accum: f32 = 0.0;
        let mut meter_peak: f32 = 0.0;
        let mut meter_count: u32 = 0;
        let meter_window = (src_rate as f32 / 30.0) as u32;

        while !stop_flag.load(Ordering::Relaxed) {
            // 200 ms max wait so we still poll the stop flag even when audio is silent.
            let wait = WaitForSingleObject(event, 200);
            if wait.0 != WAIT_OBJECT_0.0 {
                continue;
            }

            loop {
                let packet_frames = match capture_client.GetNextPacketSize() {
                    Ok(n) => n,
                    Err(e) => {
                        tracing::warn!(?e, "GetNextPacketSize");
                        break;
                    }
                };
                if packet_frames == 0 {
                    break;
                }

                let mut data_ptr: *mut u8 = std::ptr::null_mut();
                let mut num_frames: u32 = 0;
                let mut flags: u32 = 0;
                let hr = capture_client.GetBuffer(
                    &mut data_ptr,
                    &mut num_frames,
                    &mut flags,
                    None,
                    None,
                );
                if hr.is_err() {
                    tracing::warn!(?hr, "IAudioCaptureClient::GetBuffer");
                    break;
                }

                let silent = (flags & AUDCLNT_BUFFERFLAGS_SILENT.0 as u32) != 0;
                let sample_count = num_frames as usize * channels;
                let mono: Vec<f32> = if silent || data_ptr.is_null() || num_frames == 0 {
                    vec![0.0; num_frames as usize]
                } else if is_float {
                    let slice = std::slice::from_raw_parts(data_ptr as *const f32, sample_count);
                    downmix_f32(slice, channels)
                } else if bits_per_sample == 16 {
                    let slice = std::slice::from_raw_parts(data_ptr as *const i16, sample_count);
                    downmix_i16(slice, channels)
                } else {
                    // 24/32-bit PCM loopback is vanishingly rare on modern Windows mixers,
                    // so rather than plumb a third code path we just zero-fill and log.
                    tracing::warn!(bits_per_sample, "unsupported loopback PCM width");
                    vec![0.0; num_frames as usize]
                };

                if let Err(e) = capture_client.ReleaseBuffer(num_frames) {
                    tracing::warn!(?e, "ReleaseBuffer");
                }

                let resampled = resample_f32(&mono, src_rate, TARGET_RATE);
                let pcm: Vec<i16> = resampled
                    .into_iter()
                    .map(|s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
                    .collect();

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
                    // Match the cap in `capture::MAX_BUFFERED_SAMPLES` — 8 hours at
                    // 16 kHz, a leak backstop rather than a feature limit. See the
                    // comment on MAX_BUFFERED_SAMPLES in capture.rs for the history
                    // (0.12.1: lifted from 30 min to 8 h after a user lost the last
                    // 10 min of a meeting to silent truncation).
                    const LOOPBACK_CAP: usize = (TARGET_RATE as usize) * 60 * 60 * 8;
                    if buf.len() + pcm.len() <= LOOPBACK_CAP {
                        buf.extend_from_slice(&pcm);
                    } else {
                        let room = LOOPBACK_CAP.saturating_sub(buf.len());
                        if room > 0 {
                            buf.extend_from_slice(&pcm[..room]);
                        }
                        // Fire-once warning, like capture.rs's append_capped helper.
                        // Loopback uses a static AtomicBool because the closure is
                        // generated once per session via spawn_loopback_thread.
                        use std::sync::atomic::{AtomicBool, Ordering};
                        static CAP_HIT: AtomicBool = AtomicBool::new(false);
                        if !CAP_HIT.swap(true, Ordering::AcqRel) {
                            tracing::warn!(
                                "loopback: max-recording-length safety cap reached; new system-audio is being dropped. \
                                 stop and start a new recording."
                            );
                        }
                    }
                }
                let _ = frames_tx.try_send(pcm);
            }
        }

        let _ = client.Stop();
        // The WAVEFORMATEX* allocated by GetMixFormat is owned by the caller; CoTaskMemFree
        // is the correct cleanup. Skipping it is a tiny leak per-start which is acceptable
        // for a dictation session that starts/stops a handful of times.
    }
    Ok(())
}

// --- helpers (kept local to avoid pulling capture.rs internals into a pub surface) ---

fn downmix_f32(data: &[f32], channels: usize) -> Vec<f32> {
    if channels <= 1 {
        return data.to_vec();
    }
    data.chunks_exact(channels)
        .map(|frame| frame.iter().sum::<f32>() / channels as f32)
        .collect()
}

fn downmix_i16(data: &[i16], channels: usize) -> Vec<f32> {
    if channels <= 1 {
        return data.iter().map(|s| *s as f32 / i16::MAX as f32).collect();
    }
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
