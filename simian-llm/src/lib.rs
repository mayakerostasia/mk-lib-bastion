pub mod agent;
pub mod client;
pub mod error;
pub mod protocol;
pub mod registry;
pub mod spawner;
pub mod types;

pub use agent::LlmAgent;
pub use client::LlmClient;
pub use error::LlmError;
pub use protocol::{LlmChatRequest, LlmChatResponse, LlmReasonRequest, LlmReasonResponse};
pub use registry::{AgentMetadata, AgentRegistry, AgentStatus};
pub use spawner::{AgentHandle, AgentSpawner, SpawnConfig};
pub use types::{ChatMessage, ChatRole, LlmResponse, ModelInfo, TokenUsage};

// Re-export commonly used tracing/metrics macros for convenience
pub use simian_tracing::prelude::*;
pub use metrics::counter;


