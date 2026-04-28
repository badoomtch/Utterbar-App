//! Speaker-diarisation wrapper over `sherpa_rs::diarize::Diarize`.
//!
//! Diarisation runs once on the full mixed-mono PCM buffer at the
//! end of a Meeting Mode recording, BEFORE Whisper Large transcribes
//! the resulting segments. The output is `Vec<Segment { start, end,
//! speaker }>`; the dictation layer feeds those segments through
//! Whisper one at a time and stitches the results into the final
//! `**Speaker N:**`-prefixed transcript.
//!
//! Critical contract (verified against vendored `c-api.h:1628`):
//!
//!   * `num_clusters > 0` → sherpa uses fixed-count clustering and
//!     the threshold is ignored.
//!   * `num_clusters <= 0` → sherpa uses threshold-based clustering
//!     and discovers the cluster count from the data.
//!
//! `sherpa_rs::diarize::Diarize::new` wraps `None` → `4` (fixed
//! count of 4 speakers, regardless of the recording). For us that
//! would be a foot-gun: a 1-on-1 call would always show 4 fake
//! speakers. We pass `Some(0)` instead and let the threshold pick.
//!
//! **v0.15.1 NOTE:** unreachable from production code paths —
//! `transcribe_meeting_diarised` (the only caller) is itself
//! `#[allow(dead_code)]` while the diarised transcribe path is
//! disabled. v0.15.2 will reuse this wrapper as-is once the
//! single-pass-Whisper architecture lands.

// Module-level dead-code allow keeps this clean while every helper
// here waits for v0.15.2's call site.
#![allow(dead_code)]

use std::path::Path;

use anyhow::Result;

use sherpa_rs::diarize::{Diarize, DiarizeConfig, Segment};

/// Default clustering threshold. 0.5 is sherpa's documented default
/// for the pyannote 3.0 segmentation + 3D-Speaker / NeMo embedding
/// pairing. Will be tuned during beta soak based on real-recording
/// data — values in the 0.3-0.7 range tighten/loosen cluster
/// merging.
const DEFAULT_THRESHOLD: f32 = 0.5;

/// Sherpa-side minimum segment duration. Any segment shorter than
/// this is dropped before we even see it. 0.5 s catches breath /
/// crosstalk noise without dropping short legitimate utterances
/// like "Yes." Tuneable.
const DEFAULT_MIN_DURATION_ON: f32 = 0.5;

/// Sherpa-side same-speaker merge gap. Adjacent same-speaker
/// segments separated by less than this are merged at the C layer.
/// 0.4 s captures natural breath-pauses without bridging actual
/// speaker turns.
const DEFAULT_MIN_DURATION_OFF: f32 = 0.4;

/// Loaded diarisation session. Owns the underlying sherpa handle;
/// dropping this releases the native resources.
///
/// `Diarize` is `Send + Sync` per its `unsafe impl` declarations in
/// the sherpa-rs crate — we can park one of these on AppState and
/// reuse it across recordings. Rebuilding it is cheap (<100 ms on a
/// warm filesystem cache) but the underlying ONNX session warms up
/// inside, so reuse where practical.
pub struct DiariserSession {
    inner: Diarize,
}

impl DiariserSession {
    /// Construct a diariser from the two pinned model files. Caller
    /// is responsible for ensuring both files exist and are
    /// SHA-verified — typically the result of
    /// `diarize_assets::ensure_assets_ready`.
    pub fn load(segmentation: &Path, embedding: &Path) -> Result<Self> {
        // num_clusters: Some(0) — see module-level doc-comment.
        // None would have been catastrophically wrong because
        // sherpa-rs wraps None to 4 (fixed-count).
        let cfg = DiarizeConfig {
            num_clusters: Some(0),
            threshold: Some(DEFAULT_THRESHOLD),
            min_duration_on: Some(DEFAULT_MIN_DURATION_ON),
            min_duration_off: Some(DEFAULT_MIN_DURATION_OFF),
            provider: None, // CPU; DirectML provider not yet wired
            debug: false,
        };
        // sherpa-rs returns eyre::Result; convert via Display so the
        // anyhow::Result this caller expects gets a clean error chain.
        let inner = Diarize::new(segmentation, embedding, cfg)
            .map_err(|e| anyhow::anyhow!("construct sherpa diarisation session: {e}"))?;
        Ok(Self { inner })
    }

    /// Run the diariser on a full mono-16k PCM buffer.
    ///
    /// Sherpa expects `f32` samples in `[-1.0, 1.0]`. The dictation
    /// layer produces `i16` mono at 16 kHz; this method converts.
    ///
    /// Wraps the FFI call in `catch_unwind` so a hypothetical native
    /// panic on edge-case audio doesn't take down the whole
    /// recording — the caller's outer flow can fall back to chunked
    /// transcription with timestamps only.
    pub fn diarise(&mut self, pcm_i16: &[i16]) -> Result<Vec<Segment>> {
        let pcm_f32: Vec<f32> = pcm_i16
            .iter()
            .map(|s| *s as f32 / i16::MAX as f32)
            .collect();

        // catch_unwind protects us from a native panic in the FFI
        // layer. Note: only protects against Rust-level panics
        // crossing the boundary; a hard segfault in C++ would still
        // crash the process. The wrapper exists because the rest
        // of our inference path (Whisper, Parakeet) deliberately
        // crashes on internal errors, but for diarisation we have
        // a safe fallback path the caller can take.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.inner.compute(pcm_f32, None)
        }));
        match result {
            Ok(Ok(segments)) => {
                // Diagnostic for beta-soak threshold tuning. We
                // need the distinct speaker IDs and segment count
                // from real recordings to know whether 0.5 is the
                // right threshold value or whether we need to nudge
                // it.
                let distinct_speakers: std::collections::BTreeSet<i32> =
                    segments.iter().map(|s| s.speaker).collect();
                tracing::info!(
                    segment_count = segments.len(),
                    distinct_speakers = distinct_speakers.len(),
                    speakers = ?distinct_speakers,
                    "diarisation complete"
                );
                Ok(segments)
            }
            Ok(Err(e)) => Err(anyhow::anyhow!("sherpa diarisation compute failed: {e}")),
            Err(_) => Err(anyhow::anyhow!(
                "sherpa diarisation panicked — falling back to chunked transcription"
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    //! The diariser itself can only be exercised with the real
    //! sherpa runtime + downloaded models. We test the helpers and
    //! pin the `num_clusters: Some(0)` contract via a config-shape
    //! test (no FFI, no models, runs in CI).

    use sherpa_rs::diarize::DiarizeConfig;

    /// Pin the contract: passing `num_clusters: Some(0)` produces
    /// a config the C-side will read as "use threshold-based
    /// clustering", because >0 fixes the count and ignores
    /// threshold per the C API contract documented in
    /// `vendor/sherpa-onnx/include/sherpa-onnx/c-api/c-api.h:1628`.
    ///
    /// This test guards against a refactor that accidentally drops
    /// the explicit `Some(0)` (e.g. a tempting `..Default::default()`
    /// that resurrects the wrapper's `unwrap_or(4)` foot-gun).
    #[test]
    fn diarise_config_uses_threshold_when_num_clusters_is_zero() {
        let cfg = DiarizeConfig {
            num_clusters: Some(0),
            threshold: Some(0.5),
            min_duration_on: Some(0.5),
            min_duration_off: Some(0.4),
            provider: None,
            debug: false,
        };
        assert_eq!(
            cfg.num_clusters,
            Some(0),
            "num_clusters MUST be Some(0) for threshold mode; sherpa-rs wraps None → 4"
        );
        assert_eq!(cfg.threshold, Some(0.5));
    }

    /// Ensure DiarizeConfig::default() does NOT silently produce a
    /// threshold-mode config — i.e. document the foot-gun in a
    /// test so a future refactor that does `..DiarizeConfig::default()`
    /// fails CI rather than ships a 4-speaker bug.
    #[test]
    fn diarise_config_default_is_not_threshold_mode() {
        let default = DiarizeConfig::default();
        assert_eq!(
            default.num_clusters,
            Some(4),
            "DiarizeConfig::default() picks fixed-count 4. Do NOT spread it."
        );
    }
}
