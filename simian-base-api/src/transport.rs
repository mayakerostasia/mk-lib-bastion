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