use async_trait::async_trait;
use anyhow::Result;
use async_nats::Client;
use bb_lib_base_api::transport::{AcpMessage, AgentId, Transport};
use futures::stream::{BoxStream, StreamExt};
use std::fmt::Debug;

#[derive(Clone)]
pub struct NatsTransport {
    client: Client,
    // The base subject for agent communication, e.g., "agents"
    base_subject: String,
}

impl NatsTransport {
    pub async fn new(nats_url: &str, base_subject: &str) -> Result<Self> {
        let client = async_nats::connect(nats_url).await?;
        Ok(Self {
            client,
            base_subject: base_subject.to_string(),
        })
    }

    fn get_subject_for_agent(&self, agent_id: &AgentId) -> String {
        format!("{}.{}.inbox", self.base_subject, agent_id.0)
    }
}

impl Debug for NatsTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NatsTransport")
            .field("base_subject", &self.base_subject)
            .finish()
    }
}

#[async_trait]
impl Transport for NatsTransport {
    async fn publish(&self, message: AcpMessage) -> Result<()> {
        let target_subject = if let Some(ref target) = message.target {
            self.get_subject_for_agent(target)
        } else {
            format!("{}.broadcast", self.base_subject)
        };

        let payload = serde_json::to_vec(&message)?;
        self.client.publish(target_subject, payload.into()).await?;
        Ok(())
    }

    async fn subscribe(&self, agent_id: &AgentId) -> Result<BoxStream<'static, AcpMessage>> {
        let inbox_subject = self.get_subject_for_agent(agent_id);
        let broadcast_subject = format!("{}.broadcast", self.base_subject);
        
        // Subscribe to both direct inbox and broadcast
        let inbox_sub = self.client.subscribe(inbox_subject).await?;
        let broadcast_sub = self.client.subscribe(broadcast_subject).await?;

        // Merge the streams and map to AcpMessage
        let merged = futures::stream::select(inbox_sub, broadcast_sub);
        
        let acp_stream = merged.filter_map(|msg| async move {
            match serde_json::from_slice::<AcpMessage>(&msg.payload) {
                Ok(acp_msg) => Some(acp_msg),
                Err(e) => {
                    tracing::error!("Failed to deserialize AcpMessage from NATS: {}", e);
                    None
                }
            }
        });

        Ok(acp_stream.boxed())
    }

    async fn is_connected(&self) -> bool {
        self.client.connection_state() == async_nats::connection::State::Connected
    }
}