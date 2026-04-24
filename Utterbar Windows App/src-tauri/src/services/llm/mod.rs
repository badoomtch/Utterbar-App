//! Post-processing provider: local LLM (optional) or deterministic fallback.

pub mod deterministic;
pub mod llama_cpp;
pub mod provider;

pub use provider::{LlmProvider, ProcContext};

use std::sync::Arc;

pub async fn best_available() -> Arc<dyn LlmProvider> {
    #[cfg(feature = "llama")]
    {
        if let Ok(p) = llama_cpp::LlamaCppProvider::try_new().await {
            return Arc::new(p);
        }
    }
    Arc::new(deterministic::DeterministicProvider)
}
