/// Example: LlmAgent API demonstration
///
/// This example demonstrates:
/// - Creating an LlmAgent with Transport + LlmClient
/// - Sending LLM requests and broadcasts via ACP messaging
/// - Handling LLM requests from other agents
///
/// To run this example:
/// cargo run --example agent_llm_chat -p simian-llm --release

use simian_llm::{ChatMessage, LlmAgent, LlmClient, LlmResponse, LmStudioClient, LmStudioConfig};
use std::time::Duration;
use simian_base_api::transport::{AcpMessage, AgentId, Performative, Transport};
use anyhow::Result;
use async_trait::async_trait;
use futures::stream::BoxStream;
use std::sync::Arc;

/// Mock transport for demonstration (doesn't actually send over NATS)
#[derive(Clone, Debug)]
struct DemoTransport {
    messages: Arc<std::sync::Mutex<Vec<AcpMessage>>>,
}

impl DemoTransport {
    fn new() -> Self {
        Self {
            messages: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    fn show_messages(&self) {
        let messages = self.messages.lock().unwrap();
        for msg in messages.iter() {
            println!(
                "  📤 Message: {} -> {:?}",
                msg.source.0,
                msg.target.as_ref().map(|t| &t.0)
            );
            println!("     Subject: {}", msg.subject);
            println!("     Performative: {:?}", msg.performative);
        }
    }
}

#[async_trait]
impl Transport for DemoTransport {
    async fn publish(&self, message: AcpMessage) -> Result<()> {
        self.messages.lock().unwrap().push(message);
        Ok(())
    }

    async fn subscribe(&self, _agent_id: &AgentId) -> Result<BoxStream<'static, AcpMessage>> {
        Err(anyhow::anyhow!("Demo transport doesn't support subscribe"))
    }

    async fn is_connected(&self) -> bool {
        true
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🚀 LlmAgent API Demonstration\n");

    // Configuration
    let llm_url = "http://192.168.1.7:1234";
    let llm_token = "sk-lm-gLUHl47J:Cc8UhKBjQJlro3pNutt0";

    println!("📋 Configuration:");
    println!("  LMStudio: {}", llm_url);
    println!("  Agents: agent-1, agent-2\n");

    // Create LMStudio client
    println!("🤖 Initializing LMStudio client...");
    let llm_config = LmStudioConfig::new(llm_url, "default")
        .with_api_token(llm_token)
        .with_timeout(Duration::from_secs(60))
        .with_max_tokens(256);

    let llm_client = match LmStudioClient::new(llm_config).await {
        Ok(client) => {
            match client.health_check().await {
                Ok(true) => {
                    println!("✓ LMStudio is healthy\n");
                    client
                }
                _ => {
                    println!("⚠ LMStudio health check failed\n");
                    client
                }
            }
        }
        Err(e) => {
            println!("⚠ Failed to connect to LMStudio: {}", e);
            println!("  Make sure LMStudio is running at {}\n", llm_url);
            anyhow::bail!("LMStudio connection failed")
        }
    };

    // Create demo transports (would be NatsTransport in real scenario)
    println!("🔗 Creating agent transports...");
    let transport_agent1 = DemoTransport::new();
    let transport_agent2 = DemoTransport::new();
    println!("✓ Demo transports created (would be NATS in production)\n");

    // Create LLM agents
    println!("👥 Creating LLM agents...");
    let agent1 = LlmAgent::new("agent-1", transport_agent1.clone(), llm_client.clone());
    let agent2 = LlmAgent::new("agent-2", transport_agent2.clone(), llm_client.clone());
    println!("✓ Agents created\n");

    // Demo 1: Agent 1 requests completion from Agent 2
    println!("📝 Demo 1: Agent-1 requests completion from Agent-2");
    println!("  Request: 'What is Rust?'\n");

    agent1
        .request_completion("agent-2", "What is Rust?")
        .await?;

    println!("  ✓ Message published to transport");
    transport_agent1.show_messages();
    println!();

    // Demo 2: Agent 1 broadcasts a reasoning request
    println!("📡 Demo 2: Agent-1 broadcasts a reasoning request");
    println!("  Broadcast: 'Explain machine learning in one sentence.'\n");

    agent1
        .broadcast_completion("Explain machine learning in one sentence.")
        .await?;

    println!("  ✓ Broadcast message published to transport");
    transport_agent1.show_messages();
    println!();

    // Demo 3: Agent 1 sends a chat request to Agent 2
    println!("💬 Demo 3: Agent-1 sends a chat request to Agent-2");

    let messages = vec![
        ChatMessage::system("You are a helpful assistant specialized in Rust."),
        ChatMessage::user("Tell me about Rust ownership."),
    ];

    println!("  System: You are a helpful assistant specialized in Rust.");
    println!("  User: Tell me about Rust ownership.\n");

    agent1.request_chat("agent-2", &messages).await?;

    println!("  ✓ Chat message published to transport");
    transport_agent1.show_messages();
    println!();

    // Demo 4: Direct LLM calls (not via agent messaging)
    println!("🎯 Demo 4: Direct LLM calls (local execution)\n");

    println!("  Local completion call:");
    match llm_client.complete("What is the capital of France?").await {
        Ok(response) => {
            println!("    ✓ Response: {}", response.content.trim());
            if let Some(usage) = response.usage {
                println!(
                    "    Tokens: prompt={:?}, completion={:?}",
                    usage.prompt_tokens, usage.completion_tokens
                );
            }
        }
        Err(e) => println!("    ✗ Error: {}", e),
    }

    println!();
    println!("  Local chat call:");
    let chat_messages = vec![ChatMessage::user("What is 2+2?")];
    match llm_client.chat(&chat_messages).await {
        Ok(response) => {
            println!("    ✓ Response: {}", response.content.trim());
            if let Some(usage) = response.usage {
                println!(
                    "    Tokens: prompt={:?}, completion={:?}",
                    usage.prompt_tokens, usage.completion_tokens
                );
            }
        }
        Err(e) => println!("    ✗ Error: {}", e),
    }

    println!();
    println!("✅ Demo complete!\n");
    println!("📚 What just happened:");
    println!("   ✓ Created 2 LLM agents with pluggable transports");
    println!("   ✓ Published completion requests between agents");
    println!("   ✓ Demonstrated broadcast messaging (no specific target)");
    println!("   ✓ Published chat requests between agents");
    println!("   ✓ Made direct LLM calls via LmStudioClient");
    println!("   ✓ All agent messages follow the ACP standard\n");

    println!("🔗 Message Flow:");
    println!("   Agent-1 -> [Transport] -> Agent-2 (request_completion)");
    println!("   Agent-1 -> [Transport] -> All Agents (broadcast_completion)");
    println!("   Agent-1 -> [Transport] -> Agent-2 (request_chat)");
    println!("   Agent-1 -> [LMStudio] -> Direct LLM Response\n");

    println!("💡 In production:");
    println!("   - Replace DemoTransport with NatsTransport");
    println!("   - Agents subscribe to incoming messages");
    println!("   - Incoming llm.reason/llm.chat requests are auto-processed");
    println!("   - Responses flow back via the transport\n");

    Ok(())
}

