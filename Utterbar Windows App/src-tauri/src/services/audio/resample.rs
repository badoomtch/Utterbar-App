//! Stateful FFT-based downsampler for the cpal capture path.
//!
//! Why this exists: the previous resampler in `capture.rs` was a linear
//! interpolator. Linear interpolation is the cheapest and worst
//! resampler. It introduces:
//!   * aliasing distortion (no anti-aliasing low-pass filter)
//!   * smeared transients
//!   * audible frequency-dependent attenuation
//!
//! Going 48 kHz → 16 kHz via linear interpolation is exactly the
//! "tinny / muffled / underwater" character users reported in v0.14.x.
//! The hardware delivered clean broadband audio; we destroyed it on the
//! way down to 16 kHz before the recogniser ever saw it.
//!
//! `FftFixedIn` from rubato (which is already a workspace dep) does
//! proper FFT-based resampling with built-in anti-aliasing. The
//! tradeoff is it processes fixed-size chunks (`CHUNK_IN` input
//! samples per process call) so we have to buffer input across cpal
//! callbacks — cpal delivers variable-size chunks every few ms.
//!
//! Design mirrors Handy's `FrameResampler`
//! (github.com/cjpais/Handy, MIT). We pull the chunk-buffer pattern
//! and the rubato config from there because it's the same problem
//! domain (cpal capture → 16 kHz mono for ASR) and it's been soaked
//! across thousands of installs.

use rubato::{FftFixedIn, Resampler};

/// Input samples per FFT process call. 1024 is rubato's recommended
/// sweet spot: fewer = more overhead, more = more latency. At 48 kHz
/// input, 1024 samples is ~21 ms per chunk, which is comfortably
/// below cpal's typical 50-100 ms callback granularity so we're
/// never the bottleneck.
const CHUNK_IN: usize = 1024;

/// Holds the rubato resampler plus a small input ring so we can feed
/// it variable-size chunks from cpal and only process when we have a
/// full `CHUNK_IN`-sample window. `pending_input` accumulates source
/// samples between full processing windows; `pending_output` holds
/// resampled samples that haven't been drained by the caller yet.
pub struct StatefulResampler {
    /// None when source and target rates match — pass-through mode.
    inner: Option<FftFixedIn<f32>>,
    /// Source-rate samples awaiting a full CHUNK_IN window. Capacity
    /// is exactly CHUNK_IN so it never grows.
    pending_input: Vec<f32>,
    /// Resampled output samples waiting to be drained.
    pending_output: Vec<f32>,
}

impl StatefulResampler {
    pub fn new(src_rate: u32, dst_rate: u32) -> Self {
        let inner = if src_rate == dst_rate {
            None
        } else {
            // Mono only (1 channel). The 2 at the end is the
            // "sub_chunks" param — rubato splits internal work, 2 is
            // its standard recommendation. Failure here only happens
            // on absurd parameter combos (zero rates, etc.); in that
            // case we fall back to no resampling and let the caller
            // see the source rate.
            FftFixedIn::<f32>::new(src_rate as usize, dst_rate as usize, CHUNK_IN, 2, 1).ok()
        };
        Self {
            inner,
            pending_input: Vec::with_capacity(CHUNK_IN),
            pending_output: Vec::new(),
        }
    }

    /// Push variable-length f32 mono samples at the source rate.
    /// Returns resampled f32 mono samples at the destination rate.
    /// Output is whatever the FFT processor has produced since the
    /// last call — could be empty (input not yet a full window) or
    /// hundreds of samples.
    pub fn push(&mut self, samples: &[f32]) -> Vec<f32> {
        // Pass-through if rates already match.
        let resampler = match self.inner.as_mut() {
            Some(r) => r,
            None => return samples.to_vec(),
        };

        self.pending_output.clear();

        let mut src = samples;
        while !src.is_empty() {
            let space = CHUNK_IN - self.pending_input.len();
            let take = space.min(src.len());
            self.pending_input.extend_from_slice(&src[..take]);
            src = &src[take..];

            if self.pending_input.len() == CHUNK_IN {
                if let Ok(out) = resampler.process(&[&self.pending_input[..]], None) {
                    self.pending_output.extend_from_slice(&out[0]);
                }
                self.pending_input.clear();
            }
        }

        std::mem::take(&mut self.pending_output)
    }

    /// Drain any remaining buffered input, padded with zeros. Call on
    /// stop_recording so the tail of speech isn't lost. The padding
    /// adds at most one chunk of silence to the output (~21 ms at
    /// 48 kHz input), which the silence-trim post-processing will
    /// remove if `silence_removal` is on.
    ///
    /// Currently unused — the cpal stream stops cleanly on the next
    /// callback after we set `stop_flag`, so the partial chunk in
    /// `pending_input` at stop is at most ~21 ms of trailing audio
    /// that the user hasn't said anything in. Kept here so a future
    /// streaming-ASR path that needs the tail flush has a clean API.
    #[allow(dead_code)]
    pub fn finish(&mut self) -> Vec<f32> {
        let resampler = match self.inner.as_mut() {
            Some(r) => r,
            None => return Vec::new(),
        };
        if self.pending_input.is_empty() {
            return Vec::new();
        }
        self.pending_input.resize(CHUNK_IN, 0.0);
        let out = match resampler.process(&[&self.pending_input[..]], None) {
            Ok(o) => o[0].clone(),
            Err(_) => Vec::new(),
        };
        self.pending_input.clear();
        out
    }
}
