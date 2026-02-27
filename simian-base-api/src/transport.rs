use async_trait::async_trait;
use bytes::Bytes;
use futures::stream::BoxStream;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use anyhow::Result;
use uuid::Uuid;

/// Unique identifier for an Agent in the Bastion system.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AgentId(pub String);


impl AgentId {
    pub fn new(id: &str) -> Self {
        Self(id.to_string())
    }
    
    pub fn random() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

/// Performatives define the intent of an ACP message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Performative {
    Request,
    Inform,
    Query,
    Agree,
    Refuse,
    Failure,
    Subscribe,
    Unsubscribe,
}

/// The Agent Communication Protocol (ACP) message structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpMessage {
    pub message_id: Uuid,
    pub source: AgentId,
    pub target: Option<AgentId>, // None for broadcast
    pub performative: Performative,
    pub subject: String,
    pub conversation_id: Uuid,
    pub payload: Bytes,
    pub timestamp: i64,
}

/// Returns `true` if `subject` matches `pattern`.
///
/// - **Exact**: `"a.b"` matches only `"a.b"`
/// - **Single-level wildcard**: `"a.*"` matches `"a.foo"` but **not** `"a.foo.bar"`
///
/// ```
/// use simian_base_api::transport::match_subject;
/// assert!(match_subject("data.*", "data.import"));
/// assert!(!match_subject("data.*", "data.import.csv"));
/// assert!(match_subject("llm.request", "llm.request"));
/// assert!(!match_subject("llm.request", "llm.response"));
/// ```
pub fn match_subject(pattern: &str, subject: &str) -> bool {
    if let Some(prefix) = pattern.strip_suffix(".*") {
        subject.starts_with(prefix)
            && subject.len() > prefix.len() + 1
            && subject.as_bytes()[prefix.len()] == b'.'
            && !subject[prefix.len() + 1..].contains('.')
    } else {
        pattern == subject
    }
}

/// The core trait that any messaging backend (NATS, Iggy, etc.) must implement.
#[async_trait]
pub trait Transport: Send + Sync + Debug {
    /// Publishes an ACP message to the network.
    async fn publish(&self, message: AcpMessage) -> Result<()>;

    /// Subscribes to messages for this agent and returns a stream.
    async fn subscribe(&self, agent_id: &AgentId) -> Result<BoxStream<'static, AcpMessage>>;

    /// Checks if the transport is currently connected.
    async fn is_connected(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_subject_exact() {
        assert!(match_subject("llm.request", "llm.request"));
        assert!(!match_subject("llm.request", "llm.response"));
        assert!(!match_subject("llm.request", "llm.request.extra"));
    }

    #[test]
    fn test_match_subject_wildcard() {
        assert!(match_subject("data.*", "data.import"));
        assert!(match_subject("data.*", "data.export"));
        assert!(!match_subject("data.*", "data.import.csv"));
        assert!(!match_subject("data.*", "data"));
        assert!(!match_subject("data.*", "other.import"));
    }

    #[test]
    fn test_match_subject_wildcard_no_prefix_bleed() {
        // "dataX.import" should not match "data.*"
        assert!(!match_subject("data.*", "dataX.import"));
    }
}
