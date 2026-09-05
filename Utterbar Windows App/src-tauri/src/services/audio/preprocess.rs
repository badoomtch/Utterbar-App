//! Pre-ASR conditioning:
//!   * `normalise` — peak-normalise to just below full scale
//!   * `trim_silence` — drop leading/trailing silence
//!   * `suppress_noise` — DC blocker + high-pass + adaptive noise-floor
//!     gate, tuned per `NoiseSuppression` level.
//!   * `noise_gate` — standalone inter-word gate that zeros short
//!     sub-threshold blips (keyboard clicks, lip smacks, breath) without
//!     affecting broadband content.

use crate::domain::NoiseSuppression;

pub fn normalise(pcm: &mut [i16]) {
    let peak = pcm.iter().map(|s| s.abs()).max().unwrap_or(0);
    if peak == 0 || peak >= i16::MAX - 32 {
        return;
    }
    let gain = (i16::MAX as f32 - 64.0) / peak as f32;
    for s in pcm.iter_mut() {
        *s = ((*s as f32 * gain).clamp(i16::MIN as f32, i16::MAX as f32)) as i16;
    }
}

pub fn trim_silence(pcm: Vec<i16>, threshold: i16) -> Vec<i16> {
    const EDGE_PADDING_SAMPLES: usize = 3_200; // 200 ms at the 16 kHz ASR rate.

    let threshold = threshold.unsigned_abs();
    let speech_start = pcm
        .iter()
        .position(|sample| sample.unsigned_abs() > threshold)
        .unwrap_or(pcm.len());
    if speech_start == pcm.len() {
        return Vec::new();
    }

    let speech_end = pcm
        .iter()
        .rposition(|sample| sample.unsigned_abs() > threshold)
        .unwrap_or(speech_start);
    let start = speech_start.saturating_sub(EDGE_PADDING_SAMPLES);
    let end = speech_end
        .saturating_add(EDGE_PADDING_SAMPLES)
        .min(pcm.len() - 1);
    pcm[start..=end].to_vec()
}

/// Tuning constants per level. Returned tuple is
/// `(high_pass_corner_hz, floor_multiplier, absolute_min_threshold)`.
///   * `high_pass_corner_hz` — the lower the corner, the more low-end
///     rumble survives. 60 Hz preserves warmth; 140 Hz is thin but strips
///     every fan hum.
///   * `floor_multiplier` — how many times the estimated noise floor
///     counts as "speech". 1.5× is forgiving (doesn't clip quiet
///     consonants), 2.8× is aggressive.
///   * `absolute_min_threshold` — a hard minimum so a recording made in
///     an anechoic chamber still gates against tiny electrical noise.
fn level_params(level: NoiseSuppression) -> Option<(f32, f32, f32)> {
    match level {
        NoiseSuppression::Off => None,
        NoiseSuppression::Light => Some((60.0, 1.5, 80.0)),
        NoiseSuppression::Balanced => Some((90.0, 2.0, 120.0)),
        NoiseSuppression::Aggressive => Some((140.0, 2.8, 220.0)),
    }
}

/// In-place noise suppression at 16 kHz.
///
/// Pipeline:
///   1. DC blocker (removes the electrical offset some mics add).
///   2. 1st-order high-pass at the level-specific corner (60/90/140 Hz).
///   3. Noise-floor gate: estimate the quietest 20 % of 20 ms frames,
///      apply a soft threshold set N× above that floor. Sub-threshold
///      frames decay to silence over ~40 ms so the result doesn't sound
///      like a stuttering gate.
///
/// `NoiseSuppression::Off` returns immediately. The rest are ~O(n) and
/// run in well under a millisecond even for minute-long recordings —
/// safe to call inline on the stop path without a worker.
pub fn suppress_noise(pcm: &mut [i16], level: NoiseSuppression) {
    let Some((fc, floor_mult, abs_min)) = level_params(level) else {
        return;
    };
    if pcm.len() < 16 {
        return;
    }
    const SAMPLE_RATE: f32 = 16_000.0;
    // High-pass corner set by level. alpha = RC / (RC + dt) where RC = 1/(2·π·fc).
    let rc = 1.0 / (2.0 * std::f32::consts::PI * fc);
    let dt = 1.0 / SAMPLE_RATE;
    let alpha = rc / (rc + dt);

    // Two-pass: DC blocker + high-pass fused into one IIR loop. Keep intermediate floats
    // so we don't clip inside the filter.
    let mut prev_in: f32 = 0.0;
    let mut prev_out: f32 = 0.0;
    let mut filtered: Vec<f32> = Vec::with_capacity(pcm.len());
    for &s in pcm.iter() {
        let x = s as f32;
        let y = alpha * (prev_out + x - prev_in);
        filtered.push(y);
        prev_in = x;
        prev_out = y;
    }

    // Estimate noise floor from the quietest 20% of 20 ms frames. This adapts per-recording
    // so a loud environment doesn't result in over-gating speech, and a quiet one still
    // gets fan hum cut off.
    const FRAME: usize = (SAMPLE_RATE as usize) / 50; // 20 ms
    let mut frame_rms: Vec<f32> = Vec::new();
    for chunk in filtered.chunks(FRAME) {
        if chunk.is_empty() {
            continue;
        }
        let sumsq: f32 = chunk.iter().map(|v| v * v).sum();
        frame_rms.push((sumsq / chunk.len() as f32).sqrt());
    }
    if frame_rms.is_empty() {
        // Nothing to gate; write filtered output back.
        for (i, v) in filtered.iter().enumerate() {
            pcm[i] = v.clamp(i16::MIN as f32, i16::MAX as f32) as i16;
        }
        return;
    }
    let mut sorted = frame_rms.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let floor_idx = (sorted.len() as f32 * 0.2) as usize;
    let noise_floor = sorted.get(floor_idx).copied().unwrap_or(0.0);

    // Gate threshold: floor_mult × the noise floor with a small absolute minimum so
    // completely silent recordings aren't pushed to zero.
    let threshold = (noise_floor * floor_mult).max(abs_min);
    // Decay so transitions don't click — frames below threshold fade to silence over
    // ~40 ms instead of flipping off.
    let decay_per_sample: f32 = 0.997;
    let mut gain: f32 = 1.0;
    let mut idx = 0usize;
    for (fi, rms) in frame_rms.iter().enumerate() {
        let target = if *rms > threshold { 1.0 } else { 0.0 };
        let start = fi * FRAME;
        let end = (start + FRAME).min(filtered.len());
        for sample in &filtered[start..end] {
            // Single-pole glide towards the target gain.
            gain = gain * decay_per_sample + target * (1.0 - decay_per_sample);
            let v = sample * gain;
            pcm[idx] = v.clamp(i16::MIN as f32, i16::MAX as f32) as i16;
            idx += 1;
        }
    }
    // If we ran out of frames but pcm still has tail samples, zero them rather than
    // leaving stale data.
    for slot in pcm.iter_mut().skip(idx) {
        *slot = 0;
    }
}

/// Standalone inter-word noise gate. Sustained quiet runs are attenuated,
/// never erased. Short quiet regions are left intact because unvoiced
/// consonants and softly-spoken words can sit below a frame-level RMS gate.
/// This reduces steady room noise without discarding information before ASR.
pub fn noise_gate(pcm: &mut [i16]) {
    if pcm.len() < 32 {
        return;
    }
    const SAMPLE_RATE: usize = 16_000;
    const FRAME: usize = SAMPLE_RATE / 50; // 20 ms
    let frames: Vec<f32> = pcm
        .chunks(FRAME)
        .map(|c| {
            if c.is_empty() {
                0.0
            } else {
                let s: f32 = c.iter().map(|v| (*v as f32).powi(2)).sum();
                (s / c.len() as f32).sqrt()
            }
        })
        .collect();
    if frames.is_empty() {
        return;
    }
    let mut sorted = frames.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let floor = sorted[(sorted.len() * 15) / 100]; // 15th-percentile
    let threshold = (floor * 1.7).max(150.0);

    // Only reduce the centre of a sustained quiet run. The untouched edges
    // preserve transitions into and out of soft speech.
    const MIN_QUIET_RUN_FRAMES: usize = 8; // 160 ms
    const EDGE_GUARD_FRAMES: usize = 3; // 60 ms per side
    const QUIET_GAIN: f32 = 0.35;
    let is_quiet: Vec<bool> = frames.iter().map(|r| *r < threshold).collect();
    let mut run_start = 0usize;
    while run_start < is_quiet.len() {
        if !is_quiet[run_start] {
            run_start += 1;
            continue;
        }

        let mut run_end = run_start + 1;
        while run_end < is_quiet.len() && is_quiet[run_end] {
            run_end += 1;
        }

        if run_end - run_start >= MIN_QUIET_RUN_FRAMES {
            let core_start_frame = run_start + EDGE_GUARD_FRAMES;
            let core_end_frame = run_end.saturating_sub(EDGE_GUARD_FRAMES);
            if core_start_frame < core_end_frame {
                let start = core_start_frame * FRAME;
                let end = (core_end_frame * FRAME).min(pcm.len());
                let fade_samples = FRAME.min((end - start) / 2).max(1);
                for (offset, slot) in pcm[start..end].iter_mut().enumerate() {
                    let edge_distance = offset.min(end - start - offset - 1);
                    let fade = (edge_distance as f32 / fade_samples as f32).min(1.0);
                    let gain = 1.0 - (1.0 - QUIET_GAIN) * fade;
                    *slot = (*slot as f32 * gain).clamp(i16::MIN as f32, i16::MAX as f32) as i16;
                }
            }
        }

        run_start = run_end;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Synthesise `seconds` of speech-shaped audio at 16 kHz: a 130 Hz
    /// fundamental plus four formant harmonics, modulated by an envelope
    /// of 4-Hz "syllables" so the signal has both broadband content and
    /// per-frame variability. A pure sine at one frequency is a poor
    /// proxy — every frame's RMS is identical, so the percentile-floor
    /// estimators in the gate/suppressor degenerate. This signal has
    /// peaks and valleys, like speech, and survives the Aggressive
    /// high-pass corner because most of its energy is above 200 Hz.
    fn speechy_signal(seconds: f32) -> Vec<i16> {
        let n = (16_000.0 * seconds) as usize;
        (0..n)
            .map(|i| {
                let t = i as f32 / 16_000.0;
                let pi2 = 2.0 * std::f32::consts::PI;
                // Voiced excitation: fundamental + harmonics roughly
                // matching adult-male formant peaks (700, 1200, 2600 Hz).
                let s = (t * pi2 * 130.0).sin() * 0.15
                    + (t * pi2 * 700.0).sin() * 0.20
                    + (t * pi2 * 1_200.0).sin() * 0.15
                    + (t * pi2 * 2_600.0).sin() * 0.10;
                // 4 Hz syllabic envelope so the signal isn't perfectly
                // stationary — keeps the percentile-floor estimators
                // from picking the signal itself as the floor.
                let env = 0.5 + 0.5 * (t * pi2 * 4.0).sin();
                ((s * env).clamp(-1.0, 1.0) * i16::MAX as f32) as i16
            })
            .collect()
    }

    fn rms(pcm: &[i16]) -> f32 {
        if pcm.is_empty() {
            return 0.0;
        }
        let sumsq: f64 = pcm.iter().map(|s| (*s as f64).powi(2)).sum();
        ((sumsq / pcm.len() as f64).sqrt()) as f32
    }

    /// `NoiseSuppression::Off` is required to be a true no-op so the
    /// default-off install never alters the signal it didn't ask to
    /// alter.
    #[test]
    fn suppress_noise_off_is_noop() {
        let mut pcm = speechy_signal(1.0);
        let before = pcm.clone();
        suppress_noise(&mut pcm, NoiseSuppression::Off);
        assert_eq!(pcm, before);
    }

    /// Even at `Aggressive`, a clean speech-level tone must not be
    /// gated to silence — that would be the regression that breaks
    /// the "I said something but the transcript is empty" reports.
    #[test]
    fn suppress_noise_does_not_kill_speech_level_tone() {
        let mut pcm = speechy_signal(1.0);
        let before_rms = rms(&pcm);
        suppress_noise(&mut pcm, NoiseSuppression::Aggressive);
        let after_rms = rms(&pcm);
        // Allow the high-pass to attenuate somewhat but not collapse.
        assert!(
            after_rms > before_rms * 0.5,
            "speech RMS collapsed from {before_rms} to {after_rms} under Aggressive suppression"
        );
    }

    /// `noise_gate` must not silence a continuously-loud signal.
    #[test]
    fn noise_gate_does_not_kill_continuous_signal() {
        let mut pcm = speechy_signal(1.0);
        let before_rms = rms(&pcm);
        noise_gate(&mut pcm);
        let after_rms = rms(&pcm);
        assert!(
            after_rms > before_rms * 0.7,
            "continuous tone RMS collapsed from {before_rms} to {after_rms} under noise gate"
        );
    }

    #[test]
    fn trim_silence_keeps_context_around_quiet_word_boundaries() {
        let mut pcm = vec![0_i16; 8_000];
        pcm[4_000..4_800].fill(500);

        let trimmed = trim_silence(pcm, 100);

        // 800 speech samples plus 3,200 samples (200 ms) on each side.
        assert_eq!(trimmed.len(), 7_200);
        assert_eq!(trimmed[3_200], 500);
        assert_eq!(trimmed[3_999], 500);
    }

    #[test]
    fn trim_silence_handles_full_scale_negative_samples() {
        let pcm = vec![0, i16::MIN, 0];

        let trimmed = trim_silence(pcm, 100);

        assert_eq!(trimmed, vec![0, i16::MIN, 0]);
    }

    #[test]
    fn noise_gate_attenuates_sustained_quiet_audio_without_erasing_it() {
        let mut pcm = vec![120_i16; 16_000];
        let before_nonzero = pcm.iter().filter(|sample| **sample != 0).count();

        noise_gate(&mut pcm);

        let after_nonzero = pcm.iter().filter(|sample| **sample != 0).count();
        assert_eq!(after_nonzero, before_nonzero);
        assert!(pcm.iter().any(|sample| sample.abs() < 120));
        assert!(pcm.iter().all(|sample| sample.abs() >= 40));
    }

    #[test]
    fn noise_gate_leaves_short_quiet_words_untouched() {
        let mut pcm = vec![120_i16; 7 * 320]; // 140 ms, below the sustained-run gate.
        let before = pcm.clone();

        noise_gate(&mut pcm);

        assert_eq!(pcm, before);
    }

    /// `normalise` must not introduce silence either: a non-silent
    /// input always produces a non-silent output.
    #[test]
    fn normalise_preserves_non_silent_signal() {
        let mut pcm = speechy_signal(0.5);
        normalise(&mut pcm);
        let peak = pcm.iter().map(|s| s.abs()).max().unwrap_or(0);
        assert!(peak > i16::MAX / 2, "normalise dropped peak to {peak}");
    }
}
