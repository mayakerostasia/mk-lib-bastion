use serde::{Serialize, Deserialize};
use iggy::prelude::Identifier;
use iggy::prelude::Consumer;
use iggy::prelude::PollMessages;
use iggy::prelude::PollingStrategy;
use iggy::prelude::MessageClient;
use async_trait::async_trait;
use anyhow::{anyhow, Result};
use simian_base_api::transport::{AcpMessage, AgentId, Transport};
use futures::stream::{BoxStream, StreamExt};
use iggy::clients::client::IggyClient;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

pub struct IggyTransport {
    client: IggyClient,
    stream_id: Identifier,
    base_topic: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct IggyStreamId(pub u32);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct IggyTopicId(pub u32);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct IggyPartition(pub u32);

impl IggyTransport {
    pub async fn new(client: IggyClient, stream_id: Identifier, base_topic: &str) -> Result<Self> {
        Ok(Self {
            client,
            stream_id,
            base_topic: base_topic.to_string(),
        })
    }

    fn get_topic_id_for_agent(&self, agent_id: &AgentId) -> Identifier {
        Identifier::from_str_value(&format!("{}_{}", self.base_topic, agent_id.0)).unwrap()
    }

    fn get_broadcast_topic_id(&self) -> Identifier {
        Identifier::from_str_value(&format!("{}_broadcast", self.base_topic)).unwrap()
    }
}

impl Debug for IggyTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IggyTransport")
            .field("base_topic", &self.base_topic)
            .finish()
    }
}

#[async_trait]
impl Transport for IggyTransport {
    async fn publish(&self, message: AcpMessage) -> Result<()> {
        // Target Topic
        let target_topic = if let Some(ref target) = message.target {
            self.get_topic_id_for_agent(target)
        } else {
            self.get_broadcast_topic_id()
        };


        let payload = serde_json::to_vec(&message)?;
        let iggy_message = IggyMessage::from_str(payload.into());

        self.client.send_messages(&self.stream_id, &target_topic, IggyMessage::message.payload, vec![iggy_message]).await.map_err(|e| anyhow!("Iggy send error: {}", e))?;

        Ok(())
    }

    async fn subscribe(&self, agent_id: &AgentId) -> Result<BoxStream<'static, AcpMessage>> {
        let topic_id = self.get_topic_id_for_agent(agent_id);
        let broadcast_topic_id = self.get_broadcast_topic_id();
        
        let client = Arc::new(self.client.clone());
        let stream_id = self.stream_id.clone();
        
        let stream = futures::stream::unfold((0u64, 0u64), move |(inbox_offset, broadcast_offset)| {
            let client = client.clone();
            let stream_id = stream_id.clone();
            let topic_id = topic_id.clone();
            let broadcast_topic_id = broadcast_topic_id.clone();
            
            async move {
                sleep(Duration::from_millis(100)).await;
                
                let inbox_messages = client.poll_messages(&stream_id, &topic_id, &PollMessages {
                    consumer: Consumer::default(),
                    partition_id: Some(1),
                    strategy: PollingStrategy::offset(inbox_offset),
                    count: 10,
                    auto_commit: true,
                }).await.ok();

                let broadcast_messages: Vec<IggyMessage> = client.poll_messages(&stream_id, &broadcast_topic_id, &PollMessages {
                    consumer: Consumer::default(),
                    stream_id: Identifier::from_str_value("example_stream_id").unwrap(),
                    topic_id:  Identifier::from_str_value("example_topic_id").unwrap(),
                    partition_id: Some(1),
                    strategy: PollingStrategy::offset(broadcast_offset),
                    count: 10,
                    auto_commit: true,
                }).await.ok();

                let mut combined = Vec::new();
                let mut next_inbox = inbox_offset;
                let mut next_broadcast = broadcast_offset;

                if let Some(msgs) = inbox_messages {
                    for m in msgs {
                        if let Ok(acp) = serde_json::from_slice::<AcpMessage>(&m.payload) {
                            combined.push(acp);
                        }
                        next_inbox = m.offset + 1;
                    }
                }

                if let Some(msgs) = broadcast_messages {
                    for m in msgs {
                        if let Ok(acp) = serde_json::from_slice::<AcpMessage>(&m.payload) {
                            combined.push(acp);
                        }
                        next_broadcast = m.offset + 1;
                    }
                }

                Some((futures::stream::iter(combined), (next_inbox, next_broadcast)))
            }
        }).flatten();

        Ok(stream.boxed())
    }

    async fn is_connected(&self) -> bool {
        true 
    }
}

#[cfg(test)]
mod tests {
    use simian_base_api::transport::Performative;
    use super::*;
    use uuid::Uuid;
    use bytes::Bytes;

    const STREAM_ID: u32 = 1; 
    const PARTITION_ID: u32 = 1; 

    #[test]
    fn test_acp_message_serialization() {
        let msg = AcpMessage {
            message_id: Uuid::new_v4(),
            source: AgentId::new("monkey"),
            target: Some(AgentId::new("target")),
            performative: Performative::Request,
            subject: "test".to_string(),
            conversation_id: Uuid::new_v4(),
            payload: Bytes::from("hello"),
            timestamp: 123456789,
        };

        let serialized = serde_json::to_vec(&msg).unwrap();
        let deserialized: AcpMessage = serde_json::from_slice(&serialized).unwrap();

        assert_eq!(deserialized.source, msg.source);
        assert_eq!(deserialized.subject, msg.subject);
        assert_eq!(deserialized.payload, msg.payload);
    }
}
