use anyhow::Result;
use async_trait::async_trait;
use std::fmt::Debug;

use crate::types::{ChatMessage, LlmResponse, ModelInfo};

/// Core trait for LLM client implementations.
///
/// Any LLM backend (LMStudio, OpenAI, Ollama, etc.) must implement this trait.
/// Implementations should be cloneable to allow sharing across async tasks.
///
/// # Instrumentation
///
/// Implementors SHOULD emit tracing spans and metrics for observability:
///
/// **Metrics to emit:**
/// - `counter!("llm.complete.requests").increment(1)` on `complete()` calls
/// - `counter!("llm.chat.requests").increment(1)` on `chat()` calls  
/// - `counter!("llm.health_checks").increment(1)` on `health_check()` calls
/// - `counter!("llm.*.errors").increment(1)` on any errors
///
/// **Tracing to emit:**
/// - Use `#[tracing::instrument]` macro on implementation methods
/// - Log request details with `debug!()`: prompt/message count, model, context
/// - Log response details: tokens_used, response length, model
/// - Log errors with `error!()`: connection issues, timeouts, rate limits
///
/// Example:
/// ```ignore
/// #[tracing::instrument(skip(self, messages))]
/// async fn chat(&self, messages: &[ChatMessage]) -> Result<LlmResponse> {
///     counter!("llm.chat.requests").increment(1);
///     debug!(message_count = messages.len(), model = %self.model_info().name);
///     // ... implementation ...
/// }
/// ```
///
/// This enables observability through:
/// - Structured logging (with simian-tracing)
/// - Prometheus metrics (with simian-metrics)
/// - OpenTelemetry tracing (via simian-tracing)
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
