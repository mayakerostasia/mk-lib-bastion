use async_trait::async_trait;
use anyhow::{anyhow, Result};
use bb_lib_base_api::transport::{AcpMessage, AgentId, Transport};
use futures::stream::{BoxStream, StreamExt};
use iggy::client::Client;
use iggy::clients::client::IggyClient;
use iggy::messages::send_messages::{Message, SendMessages};
use iggy::identifier::Identifier;
use iggy::models::messages::PolledMessage;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

pub struct IggyTransport {
    client: IggyClient,
    stream_id: Identifier,
    // Base topic name or prefix
    base_topic: String,
}

impl IggyTransport {
    pub async fn new(client: IggyClient, stream_id: Identifier, base_topic: &str) -> Result<Self> {
        Ok(Self {
            client,
            stream_id,
            base_topic: base_topic.to_string(),
        })
    }

    fn get_topic_id_for_agent(&self, agent_id: &AgentId) -> Identifier {
        Identifier::from_str_identifier(&format!("{}_{}", self.base_topic, agent_id.0)).unwrap()
    }

    fn get_broadcast_topic_id(&self) -> Identifier {
        Identifier::from_str_identifier(&format!("{}_broadcast", self.base_topic)).unwrap()
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
        let target_topic = if let Some(ref target) = message.target {
            self.get_topic_id_for_agent(target)
        } else {
            self.get_broadcast_topic_id()
        };

        let payload = serde_json::to_vec(&message)?;
        let iggy_message = Message::from_payload(payload.into());

        self.client.send_messages(&self.stream_id, &target_topic, &mut SendMessages {
            messages: vec![iggy_message],
            partitioning: iggy::messages::send_messages::Partitioning::default(),
        }).await.map_err(|e| anyhow!("Iggy send error: {}", e))?;

        Ok(())
    }

    async fn subscribe(&self, agent_id: &AgentId) -> Result<BoxStream<'static, AcpMessage>> {
        let topic_id = self.get_topic_id_for_agent(agent_id);
        let broadcast_topic_id = self.get_broadcast_topic_id();
        
        // This is a simplified polling-based "subscription" for Iggy
        // In a real implementation, we'd handle offsets and consumer groups properly.
        let client = Arc::new(self.client.clone());
        let stream_id = self.stream_id.clone();
        
        let stream = futures::stream::unfold((0u64, 0u64), move |(inbox_offset, broadcast_offset)| {
            let client = client.clone();
            let stream_id = stream_id.clone();
            let topic_id = topic_id.clone();
            let broadcast_topic_id = broadcast_topic_id.clone();
            
            async move {
                // Simplified: poll both topics. In production, use separate tasks/merged streams.
                sleep(Duration::from_millis(100)).await;
                
                // Poll inbox
                let inbox_messages = client.poll_messages(&stream_id, &topic_id, &iggy::messages::poll_messages::PollMessages {
                    consumer: iggy::models::consumer::Consumer::default(),
                    partition_id: 1,
                    strategy: iggy::messages::poll_messages::PollingStrategy::offset(inbox_offset),
                    count: 10,
                    auto_commit: true,
                }).await.ok();

                // Poll broadcast
                let broadcast_messages = client.poll_messages(&stream_id, &broadcast_topic_id, &iggy::messages::poll_messages::PollMessages {
                    consumer: iggy::models::consumer::Consumer::default(),
                    partition_id: 1,
                    strategy: iggy::messages::poll_messages::PollingStrategy::offset(broadcast_offset),
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
        // Iggy client connectivity check varies by transport (TCP/QUIC/HTTP)
        // Simplified for this draft.
        true 
    }
}