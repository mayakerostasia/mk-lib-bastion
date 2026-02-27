use anyhow::Result;
use bytes::Bytes;
use futures::StreamExt;
use simian_base_api::agent::SimianAgent;
use simian_base_api::transport::{AgentId, Performative};
use simian_llm::backends::LmStudioClient;
use simian_llm::client::LlmClient;
use simian_nats_streams::transport::NatsTransport;
use std::env;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()))
        .init();

    let nats_url = env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    let agent_id = env::var("AGENT_ID").unwrap_or_else(|_| "llm-agent-001".to_string());
    let system_prompt = env::var("SYSTEM_PROMPT").unwrap_or_else(|_| "You are a helpful AI assistant.".to_string());

    info!("🚀 LLM Agent {} starting...", agent_id);
    info!("📝 System prompt: {}", system_prompt.chars().take(100).collect::<String>());
    
    let llm = LmStudioClient::from_env()?;
    let transport = NatsTransport::new(&nats_url, "agents").await?;
    let agent = SimianAgent::new(AgentId(agent_id.clone()), transport);
    let mut messages = agent.listen().await?;

    info!("✅ Agent {} ready and listening", agent_id);

    while let Some(msg) = messages.next().await {
        if msg.performative == Performative::Request {
            let prompt = String::from_utf8_lossy(&msg.payload);
            info!("📥 Request from {}: {}", msg.source.0, prompt.chars().take(50).collect::<String>());

            let full_prompt = format!("{}\n\nUser: {}", system_prompt, prompt);

            match llm.complete(&full_prompt).await {
                Ok(response) => {
                    info!("📤 Sending response ({} chars)", response.content.len());
                    if let Err(e) = agent.reply(&msg, Performative::Inform, Bytes::from(response.content)).await {
                        error!("Failed to send reply: {}", e);
                    }
                }
                Err(e) => {
                    error!("❌ LLM error: {}", e);
                    let error_msg = format!("Error: {}", e);
                    let _ = agent.reply(&msg, Performative::Failure, Bytes::from(error_msg)).await;
                }
            }
        }
    }

    Ok(())
}
