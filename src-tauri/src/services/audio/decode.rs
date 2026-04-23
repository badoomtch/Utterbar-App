//! Symphonia-backed file decode. Reads any supported container/codec, downmixes to mono,
//! and resamples to 16 kHz Int16 PCM for the ASR pipeline.

use std::fs::File;
use std::path::Path;

use anyhow::{anyhow, Result};
use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error as SymError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

use super::capture::TARGET_RATE;

pub struct DecodedAudio {
    pub pcm: Vec<i16>,
    pub sample_rate: u32,
    /// Source-file duration in ms. Not read by the current file-job pipeline
    /// (which reports progress by sample count) but surfaced for diagnostics.
    #[allow(dead_code)]
    pub duration_ms: u64,
}

/// Blocking decode. Safe to call from a Tokio task via `spawn_blocking`.
pub fn decode_file(path: &Path) -> Result<DecodedAudio> {
    let file = File::open(path).map_err(|e| anyhow!("open {}: {e}", path.display()))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
        .map_err(|e| anyhow!("probe: {e}"))?;

    let mut format = probed.format;
    let track = format
        .default_track()
        .ok_or_else(|| anyhow!("no default track"))?
        .clone();
    let track_id = track.id;
    let src_rate = track
        .codec_params
        .sample_rate
        .ok_or_else(|| anyhow!("unknown sample rate"))?;
    let channels = track
        .codec_params
        .channels
        .map(|c| c.count() as u16)
        .unwrap_or(1);

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| anyhow!("make decoder: {e}"))?;

    // Accumulate mono f32 at the source rate, resample once at the end.
    let mut mono: Vec<f32> = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(SymError::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(SymError::ResetRequired) => break,
            Err(e) => return Err(anyhow!("next_packet: {e}")),
        };
        if packet.track_id() != track_id {
            continue;
        }
        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(SymError::DecodeError(_)) => continue,
            Err(e) => return Err(anyhow!("decode: {e}")),
        };
        append_mono(&mut mono, decoded, channels);
    }

    let pcm = resample_linear(&mono, src_rate, TARGET_RATE);
    let i16_pcm: Vec<i16> = pcm
        .into_iter()
        .map(|s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
        .collect();
    let duration_ms = (i16_pcm.len() as u64 * 1000) / TARGET_RATE as u64;

    Ok(DecodedAudio {
        pcm: i16_pcm,
        sample_rate: TARGET_RATE,
        duration_ms,
    })
}

fn append_mono(dst: &mut Vec<f32>, buf: AudioBufferRef<'_>, channels: u16) {
    match buf {
        AudioBufferRef::F32(b) => push_mixdown(dst, &b, channels, |v| v),
        AudioBufferRef::U8(b) => push_mixdown(dst, &b, channels, |v| (v as f32 - 128.0) / 128.0),
        AudioBufferRef::U16(b) => push_mixdown(dst, &b, channels, |v| (v as f32 - 32_768.0) / 32_768.0),
        AudioBufferRef::U24(b) => push_mixdown(dst, &b, channels, |v| {
            let iv = v.inner() as i32 - (1 << 23);
            iv as f32 / (1 << 23) as f32
        }),
        AudioBufferRef::U32(b) => push_mixdown(dst, &b, channels, |v| (v as f64 - (u32::MAX as f64 / 2.0)) as f32 / (u32::MAX as f32 / 2.0)),
        AudioBufferRef::S8(b) => push_mixdown(dst, &b, channels, |v| v as f32 / 128.0),
        AudioBufferRef::S16(b) => push_mixdown(dst, &b, channels, |v| v as f32 / 32_768.0),
        AudioBufferRef::S24(b) => push_mixdown(dst, &b, channels, |v| v.inner() as f32 / (1 << 23) as f32),
        AudioBufferRef::S32(b) => push_mixdown(dst, &b, channels, |v| v as f32 / i32::MAX as f32),
        AudioBufferRef::F64(b) => push_mixdown(dst, &b, channels, |v| v as f32),
    }
}

fn push_mixdown<S, F>(
    dst: &mut Vec<f32>,
    buf: &symphonia::core::audio::AudioBuffer<S>,
    channels: u16,
    map: F,
) where
    S: symphonia::core::sample::Sample + Copy,
    F: Fn(S) -> f32,
{
    let frames = buf.frames();
    let chans = channels.max(1) as usize;
    dst.reserve(frames);
    for frame_idx in 0..frames {
        let mut acc = 0.0f32;
        for c in 0..chans {
            let plane = buf.chan(c);
            acc += map(plane[frame_idx]);
        }
        dst.push(acc / chans as f32);
    }
}

fn resample_linear(input: &[f32], src: u32, dst: u32) -> Vec<f32> {
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
