use crate::transport::{AcpMessage, AgentId, Performative, Transport};
use anyhow::Result;
use bytes::Bytes;
use futures::stream::BoxStream;
use uuid::Uuid;
use chrono::Utc;

/// A high-level Agent that uses a Transport to communicate via ACP.
pub struct BastionAgent<T: Transport> {
    id: AgentId,
    transport: T,
}

impl<T: Transport> BastionAgent<T> {
    pub fn new(id: AgentId, transport: T) -> Self {
        Self { id, transport }
    }

    pub fn id(&self) -> &AgentId {
        &self.id
    }

    /// Sends a request to a specific agent.
    pub async fn request(&self, target: AgentId, subject: &str, payload: Bytes) -> Result<Uuid> {
        let conversation_id = Uuid::new_v4();
        let message = AcpMessage {
            message_id: Uuid::new_v4(),
            source: self.id.clone(),
            target: Some(target),
            performative: Performative::Request,
            subject: subject.to_string(),
            conversation_id,
            payload,
            timestamp: Utc::now().timestamp(),
        };
        self.transport.publish(message).await?;
        Ok(conversation_id)
    }

    /// Sends an informational message (Inform performative).
    pub async fn inform(&self, target: Option<AgentId>, subject: &str, payload: Bytes) -> Result<()> {
        let message = AcpMessage {
            message_id: Uuid::new_v4(),
            source: self.id.clone(),
            target,
            performative: Performative::Inform,
            subject: subject.to_string(),
            conversation_id: Uuid::new_v4(),
            payload,
            timestamp: Utc::now().timestamp(),
        };
        self.transport.publish(message).await?;
        Ok(())
    }

    /// Replies to a previous message.
    pub async fn reply(&self, original_msg: &AcpMessage, performative: Performative, payload: Bytes) -> Result<()> {
        let message = AcpMessage {
            message_id: Uuid::new_v4(),
            source: self.id.clone(),
            target: Some(original_msg.source.clone()),
            performative,
            subject: format!("{}.reply", original_msg.subject),
            conversation_id: original_msg.conversation_id,
            payload,
            timestamp: Utc::now().timestamp(),
        };
        self.transport.publish(message).await?;
        Ok(())
    }

    /// Subscribes to messages for this agent.
    pub async fn listen(&self) -> Result<BoxStream<'static, AcpMessage>> {
        self.transport.subscribe(&self.id).await
    }
}