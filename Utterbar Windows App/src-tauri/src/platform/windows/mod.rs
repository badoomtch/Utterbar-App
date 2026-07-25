pub mod context;
pub mod startup;
pub mod text_inject;
pub mod uia_thread;

// Experimental Teams + Discord mic-mute prototype, gated behind the
// `experimental_mic_mute` Cargo feature so release builds do not link
// it. See docs/decisions/true-per-app-microphone-mute-v1-spec.md.
#[cfg(feature = "experimental_mic_mute")]
pub mod window_send_input;
