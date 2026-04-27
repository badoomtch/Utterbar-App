//! Safe Rust wrapper for sherpa-onnx's `OnlineRecognizer` C API.
//!
//! Mirrors `sherpa-rs`'s offline `TransducerRecognizer` (in the
//! upstream crate's `transducer.rs`) but talks to the *online* /
//! streaming recogniser instead. The upstream crate doesn't ship an
//! online wrapper, so we write our own here using the same
//! `sherpa-rs-sys` bindgen output.
//!
//! ## Design notes
//!
//! - `OnlineRecognizer` is the model handle. Created once per loaded
//!   model, dropped only when unloading. Heavy — owns the loaded ONNX
//!   graphs.
//! - `OnlineStream` is the per-recording state. Cheap — owns just the
//!   chunked encoder state and the decoded token sequence so far.
//!   Created fresh for each recording, dropped at the end.
//! - The C API is *thread-safe at the recognizer level* per
//!   sherpa-onnx's documentation (`OnlineRecognizer` ≈ C++
//!   `std::shared_ptr<OnlineRecognizerImpl>` internally), so we mark
//!   it `Send + Sync`. The *stream* is `Send` but not `Sync` — only
//!   one thread at a time may push waveform chunks into a given
//!   stream. That matches our usage: capture thread feeds the stream;
//!   nobody else touches it.
//! - All `CString` allocations made for config strings are kept alive
//!   across the lifetime of the recognizer. Sherpa copies them
//!   internally (per the C++ source, `OnlineRecognizerImpl::Config`
//!   stores `std::string` not `const char*`), so we could drop them
//!   immediately after `Create...`, but holding them is cheaper than
//!   debugging a use-after-free if that ever changes.
//!
//! ## API surface
//!
//! Compare to the offline path (`sherpa_rs::transducer::TransducerRecognizer`):
//!
//! Offline: `recognizer.transcribe(sample_rate, &samples) -> String`
//! Online:  `let mut stream = recognizer.create_stream();`
//!          `stream.accept_waveform(sample_rate, &chunk);`
//!          `while recognizer.is_ready(&stream) { recognizer.decode(&stream); }`
//!          `let partial = recognizer.result(&stream);`
//!          `// repeat. when done:`
//!          `stream.input_finished();`
//!          `let final = recognizer.result(&stream);`
//!
//! The orchestration belongs in the calling layer; this module only
//! exposes the primitives.

use anyhow::{anyhow, Result};
use std::ffi::{CStr, CString};
use std::mem;

use sherpa_rs_sys as sys;

/// Configuration for an online streaming recogniser.
///
/// The transducer-shaped fields (encoder/decoder/joiner/tokens) point
/// at the model files on disk. The endpoint-detection fields control
/// when sherpa decides an utterance has ended; we usually disable them
/// because Utterbar uses a hotkey-driven push-to-talk model rather
/// than VAD-driven endpointing.
#[derive(Debug, Clone)]
pub struct OnlineRecognizerConfig {
    /// Path to the streaming encoder ONNX file.
    pub encoder: String,
    /// Path to the streaming decoder ONNX file.
    pub decoder: String,
    /// Path to the streaming joiner ONNX file.
    pub joiner: String,
    /// Path to the tokens.txt file.
    pub tokens: String,

    /// Number of CPU threads for inference. 1-2 is plenty for a
    /// streaming model on modern hardware; more threads help offline
    /// batch decoding more than online chunked decoding.
    pub num_threads: i32,

    /// Audio sample rate the model was trained on. 16000 for every
    /// shipping streaming model in sherpa-onnx's catalogue.
    pub sample_rate: i32,

    /// Mel feature dimension. 80 for streaming Zipformer, same for
    /// streaming Parakeet.
    pub feature_dim: i32,

    /// `greedy_search` or `modified_beam_search`. Greedy is faster and
    /// good enough for live preview; beam is reserved for the offline
    /// refinement pass (Phase 2).
    pub decoding_method: String,

    /// Inference provider: "cpu" / "cuda" / "directml" / "coreml".
    /// Defaults to CPU because GPU providers have model-specific quirks
    /// (per the sherpa-rs upstream comment) and streaming is fast
    /// enough on CPU.
    pub provider: String,

    /// Enable sherpa's endpoint detection (auto-stop on trailing
    /// silence). Off by default — Utterbar's push-to-talk model means
    /// the user, not sherpa, decides when an utterance ends.
    pub enable_endpoint: bool,

    /// Verbose log to stderr. Off by default.
    pub debug: bool,
}

impl Default for OnlineRecognizerConfig {
    fn default() -> Self {
        Self {
            encoder: String::new(),
            decoder: String::new(),
            joiner: String::new(),
            tokens: String::new(),
            num_threads: 2,
            sample_rate: 16_000,
            feature_dim: 80,
            decoding_method: "greedy_search".into(),
            provider: "cpu".into(),
            enable_endpoint: false,
            debug: false,
        }
    }
}

/// Online streaming recogniser. Wraps a sherpa-onnx
/// `SherpaOnnxOnlineRecognizer*`.
pub struct OnlineRecognizer {
    raw: *const sys::SherpaOnnxOnlineRecognizer,
    /// CStrings whose pointers were handed to the recogniser at
    /// creation time. Kept alive for the recogniser's lifetime even
    /// though sherpa copies them internally — defence-in-depth.
    _kept_alive: Vec<CString>,
}

/// Per-recording streaming state. Wraps a
/// `SherpaOnnxOnlineStream*`. Borrows nothing from the recogniser
/// (the C API takes both pointers separately on every call), so
/// streams can outlive a particular `&` borrow on the recogniser.
pub struct OnlineStream {
    raw: *const sys::SherpaOnnxOnlineStream,
}

impl OnlineRecognizer {
    /// Build an online recogniser from a transducer model trio
    /// (encoder/decoder/joiner) plus tokens. Returns an error if any
    /// of the model files can't be loaded — sherpa returns NULL on
    /// failure with no error message, so the diagnostic is "config
    /// rejected" without specifics.
    pub fn new(cfg: OnlineRecognizerConfig) -> Result<Self> {
        // Convert all the &str -> CString upfront and keep them alive
        // in `kept_alive`. Sherpa documentation says it copies these
        // internally, but leaning on internal copy semantics is the
        // kind of contract that breaks across patch versions; holding
        // our own copies is cheap.
        let encoder = CString::new(cfg.encoder.as_str())?;
        let decoder = CString::new(cfg.decoder.as_str())?;
        let joiner = CString::new(cfg.joiner.as_str())?;
        let tokens = CString::new(cfg.tokens.as_str())?;
        let provider = CString::new(cfg.provider.as_str())?;
        let decoding_method = CString::new(cfg.decoding_method.as_str())?;
        let model_type = CString::new("transducer")?;

        // SAFETY: every pointer below points into a CString in
        // `kept_alive` that outlives this unsafe block (and indeed the
        // recogniser itself). The bindgen structs are repr(C) and we
        // populate every field that has a meaningful default.
        let raw = unsafe {
            let model_config = sys::SherpaOnnxOnlineModelConfig {
                transducer: sys::SherpaOnnxOnlineTransducerModelConfig {
                    encoder: encoder.as_ptr(),
                    decoder: decoder.as_ptr(),
                    joiner: joiner.as_ptr(),
                },
                tokens: tokens.as_ptr(),
                num_threads: cfg.num_threads,
                provider: provider.as_ptr(),
                debug: if cfg.debug { 1 } else { 0 },
                model_type: model_type.as_ptr(),
                // Fields we don't drive — zero them out. These are all
                // pointer / int fields that mean "absent" when zero.
                paraformer: mem::zeroed(),
                zipformer2_ctc: mem::zeroed(),
                modeling_unit: std::ptr::null(),
                bpe_vocab: std::ptr::null(),
                tokens_buf: std::ptr::null(),
                tokens_buf_size: 0,
                nemo_ctc: mem::zeroed(),
            };

            let recognizer_config = sys::SherpaOnnxOnlineRecognizerConfig {
                feat_config: sys::SherpaOnnxFeatureConfig {
                    sample_rate: cfg.sample_rate,
                    feature_dim: cfg.feature_dim,
                },
                model_config,
                decoding_method: decoding_method.as_ptr(),
                max_active_paths: 4,
                enable_endpoint: if cfg.enable_endpoint { 1 } else { 0 },
                rule1_min_trailing_silence: 2.4,
                rule2_min_trailing_silence: 1.2,
                rule3_min_utterance_length: 20.0,
                hotwords_file: std::ptr::null(),
                hotwords_score: 0.0,
                ctc_fst_decoder_config: mem::zeroed(),
                rule_fsts: std::ptr::null(),
                rule_fars: std::ptr::null(),
                blank_penalty: 0.0,
                hotwords_buf: std::ptr::null(),
                hotwords_buf_size: 0,
                hr: mem::zeroed(),
            };

            sys::SherpaOnnxCreateOnlineRecognizer(&recognizer_config)
        };

        if raw.is_null() {
            return Err(anyhow!(
                "SherpaOnnxCreateOnlineRecognizer returned null — check encoder/decoder/joiner/tokens paths"
            ));
        }

        Ok(Self {
            raw,
            _kept_alive: vec![
                encoder,
                decoder,
                joiner,
                tokens,
                provider,
                decoding_method,
                model_type,
            ],
        })
    }

    /// Open a fresh streaming session. Each recording gets its own
    /// stream; streams aren't reusable across recordings.
    pub fn create_stream(&self) -> Result<OnlineStream> {
        // SAFETY: `self.raw` is non-null by construction (checked in
        // `new`) and lives for the recogniser's lifetime.
        let raw = unsafe { sys::SherpaOnnxCreateOnlineStream(self.raw) };
        if raw.is_null() {
            return Err(anyhow!("SherpaOnnxCreateOnlineStream returned null"));
        }
        Ok(OnlineStream { raw })
    }

    /// Returns true if the stream has enough buffered audio that the
    /// decoder can advance. The caller drains it in a loop:
    /// `while is_ready(&stream) { decode(&stream); }`.
    pub fn is_ready(&self, stream: &OnlineStream) -> bool {
        // SAFETY: both pointers non-null by construction.
        unsafe { sys::SherpaOnnxIsOnlineStreamReady(self.raw, stream.raw) != 0 }
    }

    /// Advance the decoder by one chunk's worth of internal state.
    /// Idempotent if `is_ready` returns false; a no-op call costs
    /// roughly nothing.
    pub fn decode(&self, stream: &OnlineStream) {
        // SAFETY: both pointers non-null by construction.
        unsafe { sys::SherpaOnnxDecodeOnlineStream(self.raw, stream.raw) }
    }

    /// Fetch the current decoded text. Returns the cumulative
    /// transcript so far for this stream — newly-decoded tokens
    /// included since the last call. The caller is responsible for
    /// computing the diff (which suffix is "new") if it needs to
    /// emit per-token deltas.
    pub fn result(&self, stream: &OnlineStream) -> String {
        // SAFETY: both pointers non-null. Sherpa returns a pointer
        // into a result struct that we destroy with
        // SherpaOnnxDestroyOnlineRecognizerResult after extracting
        // the text. The text field inside the struct is a C string
        // owned by the result allocation, so we copy it out before
        // freeing.
        unsafe {
            let result_ptr = sys::SherpaOnnxGetOnlineStreamResult(self.raw, stream.raw);
            if result_ptr.is_null() {
                return String::new();
            }
            let result = &*result_ptr;
            let text = if result.text.is_null() {
                String::new()
            } else {
                CStr::from_ptr(result.text).to_string_lossy().into_owned()
            };
            sys::SherpaOnnxDestroyOnlineRecognizerResult(result_ptr);
            text
        }
    }

    /// Reset the stream's decoder state — clears buffered audio AND
    /// the decoded transcript. Useful when an utterance ends and the
    /// caller wants to start a new one without creating a fresh
    /// stream (which would require re-loading some per-stream state).
    pub fn reset(&self, stream: &OnlineStream) {
        unsafe { sys::SherpaOnnxOnlineStreamReset(self.raw, stream.raw) }
    }
}

impl OnlineStream {
    /// Push a chunk of f32 PCM samples (range -1.0..=1.0). Sherpa
    /// internally buffers and the recogniser advances when
    /// `is_ready` returns true. The chunk size is at the caller's
    /// discretion; 100-500 ms chunks are typical for a UI-responsive
    /// streaming experience.
    pub fn accept_waveform(&mut self, sample_rate: u32, samples: &[f32]) {
        // SAFETY: stream pointer non-null. The C side reads the
        // samples slice immediately and copies into its own buffer;
        // it does not retain the pointer past this call.
        unsafe {
            sys::SherpaOnnxOnlineStreamAcceptWaveform(
                self.raw,
                sample_rate as i32,
                samples.as_ptr(),
                samples.len() as i32,
            );
        }
    }

    /// Tell the recogniser no more audio will arrive on this stream.
    /// The decoder drains its trailing buffer; the next `result()`
    /// call returns the final transcript.
    pub fn input_finished(&mut self) {
        unsafe { sys::SherpaOnnxOnlineStreamInputFinished(self.raw) }
    }
}

// SAFETY: sherpa-onnx's OnlineRecognizer is internally thread-safe
// (uses std::shared_ptr<OnlineRecognizerImpl> in C++ with const
// methods). Multiple streams can be created and consumed concurrently.
unsafe impl Send for OnlineRecognizer {}
unsafe impl Sync for OnlineRecognizer {}

// SAFETY: an OnlineStream owns a unique pointer to a per-stream state
// object. It can be moved across threads (Send) but only one thread
// may push samples / read results at a time (no Sync). This matches
// our usage: the capture thread holds the stream and is the only
// writer; the consumer reads results from the same thread on a tick.
unsafe impl Send for OnlineStream {}

impl Drop for OnlineRecognizer {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            // SAFETY: raw was allocated by SherpaOnnxCreateOnlineRecognizer
            // and hasn't been destroyed elsewhere — Drop runs once.
            unsafe { sys::SherpaOnnxDestroyOnlineRecognizer(self.raw) }
        }
    }
}

impl Drop for OnlineStream {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            // SAFETY: raw was allocated by SherpaOnnxCreateOnlineStream
            // and hasn't been destroyed elsewhere — Drop runs once.
            unsafe { sys::SherpaOnnxDestroyOnlineStream(self.raw) }
        }
    }
}
