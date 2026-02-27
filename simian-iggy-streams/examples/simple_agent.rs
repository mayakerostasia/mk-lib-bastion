//! Simple Agent Example with IggyTransport
//!
//! This example demonstrates:
//! - Creating agents with IggyTransport (using Arc for sharing)
//! - Setting up two agents that communicate
//! - Publishing messages and listening for responses
//! - Working with Iggy's topic naming scheme
//!
//! How to run:
//! 1. Start Iggy server: `docker run -it --rm -p 3000:3000 -p 8090:8090 iggydata/iggy:latest`
//! 2. Run this example: `cargo run --example simple_agent`

use simian_iggy_streams::{IggyTransport, IggyTransportConfig};
use simian_base_api::transport::{AcpMessage, AgentId, Performative};
use iggy::clients::client::IggyClient;
use iggy::prelude::Identifier;
use anyhow::Result;
use bytes::Bytes;
use uuid::Uuid;
use futures::stream::StreamExt;
use tokio::time::{sleep, Duration};
use std::env;
use std::sync::Arc;

/// Represents a simple agent that can send and receive messages
struct SimpleAgent {
    id: AgentId,
    transport: Arc<IggyTransport>,
}

impl SimpleAgent {
    /// Create a new agent
    async fn new(agent_name: &str, transport: Arc<IggyTransport>) -> Result<Self> {
        println!("Creating agent: {}", agent_name);
        Ok(Self {
            id: AgentId::new(agent_name),
            transport,
        })
    }

    /// Send a message to another agent
    async fn send_to(&self, target: &AgentId, subject: &str, payload: &str) -> Result<()> {
        let msg = AcpMessage {
            message_id: Uuid::new_v4(),
            source: self.id.clone(),
            target: Some(target.clone()),
            performative: Performative::Request,
            subject: subject.to_string(),
            conversation_id: Uuid::new_v4(),
            payload: Bytes::from(payload.to_string()),
            timestamp: chrono::Local::now().timestamp_millis(),
        };

        println!(
            "[{}] Sending message to {}: '{}' - {}",
            self.id.0, target.0, subject, payload
        );
        self.transport.publish(msg).await?;
        Ok(())
    }

    /// Broadcast a message to all agents
    async fn broadcast(&self, subject: &str, payload: &str) -> Result<()> {
        let msg = AcpMessage {
            message_id: Uuid::new_v4(),
            source: self.id.clone(),
            target: None,  // No target = broadcast
            performative: Performative::Inform,
            subject: subject.to_string(),
            conversation_id: Uuid::new_v4(),
            payload: Bytes::from(payload.to_string()),
            timestamp: chrono::Local::now().timestamp_millis(),
        };

        println!(
            "[{}] Broadcasting message: '{}' - {}",
            self.id.0, subject, payload
        );
        self.transport.publish(msg).await?;
        Ok(())
    }

    /// Start listening for messages with a timeout
    async fn listen(&self, timeout_secs: u64) -> Result<Vec<AcpMessage>> {
        println!("[{}] Starting to listen for messages...", self.id.0);
        
        let mut messages = Vec::new();
        let mut stream = self.transport.subscribe(&self.id).await?;

        let result = tokio::time::timeout(
            Duration::from_secs(timeout_secs),
            async {
                while let Some(msg) = stream.next().await {
                    messages.push(msg);
                }
            },
        )
        .await;

        match result {
            Ok(_) => println!("[{}] Listening completed", self.id.0),
            Err(_) => println!("[{}] Listening timeout (received {} messages)", self.id.0, messages.len()),
        }

        Ok(messages)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Simple Agent Example with IggyTransport ===\n");

    // Get Iggy address from environment or use default
    let iggy_addr = env::var("IGGY_ADDR")
        .unwrap_or_else(|_| "http://localhost:8090".to_string());
    println!("Connecting to Iggy at: {}\n", iggy_addr);

    // Connect to Iggy
    let client = IggyClient::new()
        .tcp(&iggy_addr)
        .and_then(|client| client.connect())
        .await?;
    println!("✓ Connected to Iggy\n");

    // Create/verify stream
    let stream_id = Identifier::from_str_value("agents_stream")?;
    match client.get_stream(&stream_id).await {
        Ok(_) => println!("✓ Using existing stream: agents_stream"),
        Err(_) => {
            println!("✓ Creating new stream: agents_stream");
            client.create_stream(&stream_id, None).await?;
        }
    }

    // Configure transport with custom settings
    let config = IggyTransportConfig {
        poll_interval_ms: 100,      // Poll every 100ms
        batch_size: 10,              // Fetch up to 10 messages per poll
        partitions: 1,               // 1 partition per topic (simple setup)
        consumer_group: None,        // No consumer group for this example
    };

    // Create transport (wrapped in Arc for sharing across agents)
    let transport = Arc::new(IggyTransport::new_with_config(
        client,
        stream_id,
        "agents",  // base_topic
        config,
    )
    .await?);
    println!("✓ Created IggyTransport\n");

    // Create agents with transport references
    let alice = SimpleAgent::new("alice", transport.clone()).await?;
    let bob = SimpleAgent::new("bob", transport.clone()).await?;
    println!();

    // Spawn listener tasks for both agents
    let alice_id = alice.id.clone();
    let alice_transport = alice.transport.clone();
    let alice_listener = tokio::spawn(async move {
        let agent = SimpleAgent { id: alice_id, transport: alice_transport };
        agent.listen(3).await
    });

    let bob_id = bob.id.clone();
    let bob_transport = bob.transport.clone();
    let bob_listener = tokio::spawn(async move {
        let agent = SimpleAgent { id: bob_id, transport: bob_transport };
        agent.listen(3).await
    });

    // Give listeners time to start and subscribe
    sleep(Duration::from_millis(500)).await;

    // Demonstrate message flow
    println!("\n--- Message Flow ---\n");

    // Alice sends to Bob
    alice.send_to(&bob.id, "hello", "Hi Bob, how are you?").await?;
    sleep(Duration::from_millis(200)).await;

    // Bob sends to Alice
    bob.send_to(&alice.id, "reply", "I'm doing great!").await?;
    sleep(Duration::from_millis(200)).await;

    // Bob broadcasts
    bob.broadcast("status_update", "All systems operational").await?;
    sleep(Duration::from_millis(200)).await;

    // Alice also broadcasts
    alice.broadcast("greeting", "Hello everyone!").await?;
    sleep(Duration::from_millis(500)).await;

    // Wait for listeners to complete
    let alice_msgs = alice_listener.await??;
    let bob_msgs = bob_listener.await??;

    // Display received messages
    println!("\n--- Messages Received ---\n");

    println!("[alice] Received {} messages:", alice_msgs.len());
    for (i, msg) in alice_msgs.iter().enumerate() {
        println!(
            "  {}. From '{}': '{}' - {}",
            i + 1,
            msg.source.0,
            msg.subject,
            String::from_utf8_lossy(&msg.payload)
        );
    }

    println!("\n[bob] Received {} messages:", bob_msgs.len());
    for (i, msg) in bob_msgs.iter().enumerate() {
        println!(
            "  {}. From '{}': '{}' - {}",
            i + 1,
            msg.source.0,
            msg.subject,
            String::from_utf8_lossy(&msg.payload)
        );
    }

    println!("\n--- Topic Architecture ---\n");
    println!("Topics created automatically:");
    println!("  agents_alice_inbox    - Messages directed to Alice");
    println!("  agents_bob_inbox      - Messages directed to Bob");
    println!("  agents_broadcast      - Broadcast messages for all agents");
    println!("\nEach topic tracks message offsets independently");
    println!("Messages are persistent and can be replayed");

    println!("\n=== Configuration Details ===\n");
    println!("poll_interval_ms: 100   - Poll every 100ms for new messages");
    println!("batch_size: 10          - Fetch up to 10 messages per poll");
    println!("partitions: 1           - Each topic has 1 partition");
    println!("consumer_group: None    - Each agent is independent (no consumer group)");

    Ok(())
}
