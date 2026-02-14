use anyhow::Result;
use async_trait::async_trait;
use std::fmt::Debug;

use crate::types::{ChatMessage, LlmResponse, ModelInfo};

/// Core trait for LLM client implementations.
///
/// Any LLM backend (LMStudio, OpenAI, Ollama, etc.) must implement this trait.
/// Implementations should be cloneable to allow sharing across async tasks.
#[async_trait]
pub trait LlmClient: Send + Sync + Clone + Debug {
    /// Sends a simple text prompt and returns the completion.
    async fn complete(&self, prompt: &str) -> Result<LlmResponse>;

    /// Sends a chat conversation and returns the assistant's response.
    async fn chat(&self, messages: &[ChatMessage]) -> Result<LlmResponse>;

    /// Checks connectivity and availability of the LLM backend.
    async fn health_check(&self) -> Result<bool>;

    /// Returns metadata about the configured model.
    fn model_info(&self) -> &ModelInfo;
}
