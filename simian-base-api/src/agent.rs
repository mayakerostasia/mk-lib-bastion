use crate::transport::{AcpMessage, AgentId, Performative, Transport};
use anyhow::Result;
use bytes::Bytes;
use futures::stream::BoxStream;
use uuid::Uuid;
use chrono::Utc;

/// A high-level Agent that uses a Transport to communicate via ACP.
pub struct SimianAgent<T: Transport> {
    id: AgentId,
    transport: T,
}

impl<T: Transport> SimianAgent<T> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use futures::StreamExt;
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Default)]
    struct MockTransport {
        published: Arc<Mutex<Vec<AcpMessage>>>,
    }

    #[async_trait]
    impl Transport for MockTransport {
        async fn publish(&self, message: AcpMessage) -> Result<()> {
            self.published.lock().unwrap().push(message);
            Ok(())
        }
        async fn subscribe(&self, _agent_id: &AgentId) -> Result<BoxStream<'static, AcpMessage>> {
            Ok(futures::stream::empty().boxed())
        }
        async fn is_connected(&self) -> bool { true }
    }

    #[tokio::test]
    async fn test_agent_request() {
        let transport = MockTransport::default();
        let agent = SimianAgent::new(AgentId::new("sender"), transport.clone());
        let target = AgentId::new("receiver");
        
        let conv_id = agent.request(target.clone(), "test", Bytes::from("hello")).await.unwrap();
        
        let published = transport.published.lock().unwrap();
        assert_eq!(published.len(), 1);
        let msg = &published[0];
        assert_eq!(msg.source.0, "sender");
        assert_eq!(msg.target.as_ref().unwrap().0, "receiver");
        assert_eq!(msg.performative, Performative::Request);
        assert_eq!(msg.conversation_id, conv_id);
    }

    #[tokio::test]
    async fn test_agent_reply() {
        let transport = MockTransport::default();
        let agent = SimianAgent::new(AgentId::new("responder"), transport.clone());
        
        let original_msg = AcpMessage {
            message_id: Uuid::new_v4(),
            source: AgentId::new("requester"),
            target: Some(AgentId::new("responder")),
            performative: Performative::Request,
            subject: "hello".to_string(),
            conversation_id: Uuid::new_v4(),
            payload: Bytes::from("ping"),
            timestamp: Utc::now().timestamp(),
        };

        agent.reply(&original_msg, Performative::Agree, Bytes::from("pong")).await.unwrap();

        let published = transport.published.lock().unwrap();
        assert_eq!(published.len(), 1);
        let msg = &published[0];
        assert_eq!(msg.source.0, "responder");
        assert_eq!(msg.target.as_ref().unwrap().0, "requester");
        assert_eq!(msg.performative, Performative::Agree);
        assert_eq!(msg.conversation_id, original_msg.conversation_id);
        assert_eq!(msg.subject, "hello.reply");
    }
}