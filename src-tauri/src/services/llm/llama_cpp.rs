//! Optional llama.cpp-backed provider. Compiled only with `--features llama`.
//!
//! Scaffolded: the selector only ever returns the deterministic provider today.
//! Keeping the stub so enabling `--features llama` is a one-line change in the
//! selector rather than a rewrite.

#![allow(dead_code)]

#[cfg(feature = "llama")]
mod real {
    use anyhow::Result;
    use async_trait::async_trait;

    use crate::services::llm::provider::{LlmProvider, ProcContext};

    pub struct LlamaCppProvider;

    impl LlamaCppProvider {
        pub async fn try_new() -> Result<Self> {
            // TODO(I-005): locate GGUF under %APPDATA%/Utterbar/models/llm/*, load via `llama-cpp-2`.
            anyhow::bail!("llama_cpp provider pending implementation")
        }
    }

    #[async_trait]
    impl LlmProvider for LlamaCppProvider {
        fn id(&self) -> &str { "llama-cpp" }
        async fn is_available(&self) -> bool { true }
        async fn rewrite(&self, input: &str, _system: &str, _ctx: &ProcContext) -> Result<String> {
            Ok(input.into())
        }
    }
}

#[cfg(feature = "llama")]
pub use real::LlamaCppProvider;

#[cfg(not(feature = "llama"))]
pub struct LlamaCppProvider;
#[cfg(not(feature = "llama"))]
impl LlamaCppProvider {
    pub async fn try_new() -> anyhow::Result<Self> {
        anyhow::bail!("llama feature not enabled")
    }
}
