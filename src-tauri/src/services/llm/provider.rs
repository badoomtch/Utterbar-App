use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProcContext {
    pub app_exe: Option<String>,
    pub app_title: Option<String>,
    pub selection: Option<String>,
    pub clipboard: Option<String>,
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Provider id for telemetry and history attribution. Only the deterministic
    /// provider is ever instantiated today — this trait is kept so the optional
    /// `llama` feature can slot in as a second implementor without changing call sites.
    #[allow(dead_code)]
    fn id(&self) -> &str;
    /// Availability probe. Same rationale — unused in the current single-provider
    /// path but part of the contract any future provider must implement.
    #[allow(dead_code)]
    async fn is_available(&self) -> bool;
    async fn rewrite(&self, input: &str, system: &str, ctx: &ProcContext) -> Result<String>;
}
