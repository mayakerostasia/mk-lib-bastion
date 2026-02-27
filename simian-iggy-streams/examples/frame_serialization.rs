//! Frame Serialization Example with IggyTransport
//!
//! This example demonstrates:
//! - Creating an Iggy stream and topics
//! - Using IggyTransport for message serialization
//! - Publishing and consuming messages through Iggy
//!
//! Differences from NATS version:
//! - Uses Iggy's pull-based polling instead of push-based subscriptions
//! - Requires explicit stream/topic creation (done automatically here)
//! - Messages are persistent with offset-based tracking
//! - Topic naming follows: {base_topic}_{agent_id}_inbox pattern

use simian_iggy_streams::IggyTransport;
use simian_base_api::transport::{AcpMessage, AgentId, Performative};
use iggy::clients::client::IggyClient;
use iggy::prelude::Identifier;
use anyhow::Result;
use bytes::Bytes;
use uuid::Uuid;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    println!("Starting Frame Serialization Example with IggyTransport\n");

    // Get Iggy address from environment or use default
    let iggy_addr = env::var("IGGY_ADDR")
        .unwrap_or_else(|_| "http://localhost:8090".to_string());
    println!("Connecting to Iggy at: {}", iggy_addr);

    // Connect to Iggy
    let client = IggyClient::new()
        .tcp(&iggy_addr)
        .and_then(|client| client.connect())
        .await?;
    println!("Connected to Iggy!\n");

    // Create or get the stream identifier
    let stream_id = Identifier::from_str_value("agents_stream")?;
    
    // Create the stream if it doesn't exist
    match client.get_stream(&stream_id).await {
        Ok(_) => println!("Using existing stream: agents_stream"),
        Err(_) => {
            println!("Creating new stream: agents_stream");
            client.create_stream(&stream_id, None).await?;
        }
    }

    // Create the transport with default configuration
    let transport = IggyTransport::new(
        client,
        stream_id,
        "agents",  // base_topic - will create agents_alice_inbox, agents_bob_inbox, agents_broadcast
    )
    .await?;
    println!("Created IggyTransport with base_topic='agents'\n");

    // Create sample ACP messages for serialization testing
    let alice_id = AgentId::new("alice");
    let bob_id = AgentId::new("bob");
    let conversation_id = Uuid::new_v4();

    // Message 1: Alice sends a request to Bob
    let msg1 = AcpMessage {
        message_id: Uuid::new_v4(),
        source: alice_id.clone(),
        target: Some(bob_id.clone()),
        performative: Performative::Request,
        subject: "greeting".to_string(),
        conversation_id,
        payload: Bytes::from("Hello Bob!"),
        timestamp: chrono::Local::now().timestamp_millis(),
    };

    println!("Message 1 (Alice -> Bob):");
    println!("  Type: Request");
    println!("  Subject: {}", msg1.subject);
    println!("  Payload: {}", String::from_utf8_lossy(&msg1.payload));
    
    // Publish the message through Iggy
    println!("  Publishing to: agents_bob_inbox\n");
    transport.publish(msg1.clone()).await?;

    // Message 2: Bob broadcasts an inform message (no specific target)
    let msg2 = AcpMessage {
        message_id: Uuid::new_v4(),
        source: bob_id.clone(),
        target: None,  // No target means broadcast
        performative: Performative::Inform,
        subject: "status".to_string(),
        conversation_id,
        payload: Bytes::from("I'm online and ready"),
        timestamp: chrono::Local::now().timestamp_millis(),
    };

    println!("Message 2 (Bob broadcasts):");
    println!("  Type: Inform");
    println!("  Subject: {}", msg2.subject);
    println!("  Payload: {}", String::from_utf8_lossy(&msg2.payload));
    println!("  Publishing to: agents_broadcast\n");
    transport.publish(msg2.clone()).await?;

    // Message 3: Bob replies to Alice
    let msg3 = AcpMessage {
        message_id: Uuid::new_v4(),
        source: bob_id.clone(),
        target: Some(alice_id.clone()),
        performative: Performative::Reply,
        subject: "greeting".to_string(),
        conversation_id,
        payload: Bytes::from("Hello Alice! Nice to hear from you."),
        timestamp: chrono::Local::now().timestamp_millis(),
    };

    println!("Message 3 (Bob -> Alice):");
    println!("  Type: Reply");
    println!("  Subject: {}", msg3.subject);
    println!("  Payload: {}", String::from_utf8_lossy(&msg3.payload));
    println!("  Publishing to: agents_alice_inbox\n");
    transport.publish(msg3.clone()).await?;

    println!("All messages published successfully!\n");

    // Now demonstrate subscribing and receiving messages
    println!("Subscribing to messages for Alice...\n");
    
    // Alice subscribes to her inbox and broadcast messages
    let mut alice_messages = transport.subscribe(&alice_id).await?;
    
    // Use a timeout to avoid blocking forever in this example
    let timeout = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        tokio::pin!(async {
            let mut count = 0;
            while let Some(msg) = futures::StreamExt::next(&mut alice_messages).await {
                count += 1;
                println!("Alice received message {}:", count);
                println!("  From: {}", msg.source.0);
                println!("  Performative: {:?}", msg.performative);
                println!("  Subject: {}", msg.subject);
                println!("  Payload: {}", String::from_utf8_lossy(&msg.payload));
                println!();
                
                if count >= 2 {  // Expect to receive msg2 (broadcast) and msg3 (direct)
                    break;
                }
            }
        })
    ).await;

    match timeout {
        Ok(_) => println!("Successfully received and processed messages!"),
        Err(_) => println!("Timeout waiting for messages (this is normal in example)"),
    }

    println!("\n=== Key Differences from NATS ===");
    println!("1. Pull-based polling: Iggy uses poll_messages() instead of NATS push subscriptions");
    println!("2. Persistent storage: All messages are stored by Iggy with offset tracking");
    println!("3. Stream/Topic setup: Topics must be created (auto-created in this example)");
    println!("4. Polling interval: Configurable via IggyTransportConfig.poll_interval_ms");
    println!("5. Consumer offsets: Tracked per topic with auto-commit capability");
    println!("6. Topic naming: Uses format {{base_topic}}_{{agent_id}}_inbox for agent inboxes");
    println!("7. Broadcast: Uses {{base_topic}}_broadcast for fanout messages");

    Ok(())
}
