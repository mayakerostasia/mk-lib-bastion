pub mod client;
pub mod error;
pub mod protocol;
pub mod types;

pub use client::LlmClient;
pub use error::LlmError;
pub use protocol::{LlmChatRequest, LlmChatResponse, LlmReasonRequest, LlmReasonResponse};
pub use types::{ChatMessage, ChatRole, LlmResponse, ModelInfo, TokenUsage};

// Re-export commonly used tracing/metrics macros for convenience
pub use simian_tracing::prelude::*;
pub use metrics::counter;


