//! Streaming ASR primitives.
//!
//! The offline path in `services::inference` calls
//! `TransducerRecognizer::transcribe(&samples)` with the entire audio
//! buffer at once. That allocates O(duration) memory inside ONNX
//! Runtime and aborts on long inputs (see the v0.12.0 crash dump and
//! STREAMING_PLAN.md for the diagnosis). Streaming is the structural
//! fix.
//!
//! This module contains the safe Rust wrapper for sherpa-onnx's
//! `OnlineRecognizer` C API. The transitive `sherpa-rs-sys` crate
//! generates bindgen output for every streaming symbol we need; we
//! just wrap it with proper RAII, lifetimes, and Send/Sync markers.
//!
//! A single `OnlineRecognizer` is created per loaded model and shared
//! across recordings. Each recording opens a fresh `OnlineStream`,
//! pushes audio into it as it arrives from the capture callback, and
//! drains decoded tokens via `result()` periodically. The stream is
//! dropped when the recording finishes.
//!
//! Pattern mirrors sherpa-rs's offline `TransducerRecognizer` for
//! consistency.

mod online_recognizer;

pub use online_recognizer::{OnlineRecognizer, OnlineRecognizerConfig, OnlineStream};
