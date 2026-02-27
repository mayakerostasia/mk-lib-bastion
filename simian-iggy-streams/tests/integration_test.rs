//! Integration tests for IggyTransport
//!
//! These tests verify the core functionality of the IggyTransport implementation:
//! - Publish/subscribe message flow between agents
//! - Offset management and message ordering
//! - Broadcast messaging
//! - Connection status
//!
//! ## Setup Requirements
//!
//! These tests require a running Iggy server. You can start one using Docker:
//!
//! ```bash
//! docker run -d \
//!   --name iggy-server \
//!   -p 8090:8090 \
//!   -p 8091:8091 \
//!   -p 8092:8092 \
//!   ghcr.io/iggy-rs/iggy:latest
//! ```
//!
//! The server should be accessible at `127.0.0.1:8090` by default.
//!
//! To stop the server:
//! ```bash
//! docker stop iggy-server && docker rm iggy-server
//! ```

use bytes::Bytes;
use futures::StreamExt;
use simian_base_api::agent::SimianAgent;
use simian_base_api::transport::{AgentId, Performative};
use simian_iggy_streams::transport::IggyTransport;
use iggy::prelude::Identifier;
use iggy::clients::client::IggyClient;
use std::time::Duration;

/// Test helper: Create an Iggy client connected to the local server
fn create_test_client() -> IggyClient {
    IggyClient::default()
}

/// Test helper: Ensure test stream exists
async fn ensure_test_stream(client: &IggyClient, stream_id: &Identifier) -> anyhow::Result<()> {
    match client.get_stream(stream_id).await {
        Ok(_) => {
            tracing::debug!("Stream {:?} already exists", stream_id);
        }
        Err(_) => {
            tracing::info!("Creating stream {:?}", stream_id);
            client
                .create_stream(stream_id, None)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create stream: {}", e))?;
        }
    }
    Ok(())
}

#[tokio::test]
async fn test_publish_subscribe_direct_message() {
    // Initialize tracing for debugging
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();

    let client = create_test_client();
    let stream_id = Identifier::from_str_value("test_direct_msg_stream").unwrap();

    // Ensure stream exists
    ensure_test_stream(&client, &stream_id)
        .await
        .expect("Failed to ensure test stream");

    // Create transports
    let transport1 = IggyTransport::new(
        client.clone(),
        stream_id.clone(),
        "test_direct",
    )
    .await
    .expect("Failed to create transport 1");

    let transport2 = IggyTransport::new(
        client.clone(),
        stream_id.clone(),
        "test_direct",
    )
    .await
    .expect("Failed to create transport 2");

    // Create agents
    let agent_a = SimianAgent::new(AgentId::new("agent_a"), transport1);
    let agent_b = SimianAgent::new(AgentId::new("agent_b"), transport2);

    // Subscribe agent B first (to ensure it's ready before agent A sends)
    let mut inbox_b = agent_b
        .listen()
        .await
        .expect("agent_b failed to subscribe");

    // Give subscription time to propagate
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Agent A sends a direct message to Agent B
    let test_payload = Bytes::from("Hello from agent_a!");
    let conversation_id = agent_a
        .request(
            AgentId::new("agent_b"),
            "greeting",
            test_payload.clone(),
        )
        .await
        .expect("agent_a failed to send request");

    // Agent B should receive the message
    let received = tokio::time::timeout(Duration::from_secs(10), inbox_b.next())
        .await
        .expect("agent_b timed out waiting for message")
        .expect("agent_b stream ended unexpectedly");

    // Verify message properties
    assert_eq!(received.source.0, "agent_a", "Message should be from agent_a");
    assert_eq!(
        received.subject, "greeting",
        "Message subject should be 'greeting'"
    );
    assert_eq!(
        received.payload, test_payload,
        "Payload should match sent data"
    );
    assert_eq!(
        received.conversation_id, conversation_id,
        "Conversation ID should match"
    );
    assert_eq!(
        received.performative,
        Performative::Request,
        "Performative should be Request"
    );

    println!("✓ Direct message publish/subscribe test passed");
}

#[tokio::test]
async fn test_broadcast_message() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();

    let client = create_test_client();
    let stream_id = Identifier::from_str_value("test_broadcast_stream").unwrap();

    ensure_test_stream(&client, &stream_id)
        .await
        .expect("Failed to ensure test stream");

    // Create multiple agents/transports
    let transport1 = IggyTransport::new(
        client.clone(),
        stream_id.clone(),
        "test_broadcast",
    )
    .await
    .expect("Failed to create transport 1");

    let transport2 = IggyTransport::new(
        client.clone(),
        stream_id.clone(),
        "test_broadcast",
    )
    .await
    .expect("Failed to create transport 2");

    let transport3 = IggyTransport::new(
        client.clone(),
        stream_id.clone(),
        "test_broadcast",
    )
    .await
    .expect("Failed to create transport 3");

    let agent1 = SimianAgent::new(AgentId::new("broadcast_agent1"), transport1);
    let agent2 = SimianAgent::new(AgentId::new("broadcast_agent2"), transport2);
    let agent3 = SimianAgent::new(AgentId::new("broadcast_agent3"), transport3);

    // Agents 2 and 3 subscribe first
    let mut inbox2 = agent2
        .listen()
        .await
        .expect("agent2 failed to subscribe");
    let mut inbox3 = agent3
        .listen()
        .await
        .expect("agent3 failed to subscribe");

    // Give subscriptions time to propagate
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Agent 1 sends a broadcast message (no target)
    let broadcast_payload = Bytes::from("Broadcast from agent1!");
    agent1
        .broadcast("announcement", broadcast_payload.clone())
        .await
        .expect("agent1 failed to send broadcast");

    // Agent 2 should receive it
    let msg2 = tokio::time::timeout(Duration::from_secs(10), inbox2.next())
        .await
        .expect("agent2 timed out waiting for broadcast")
        .expect("agent2 stream ended unexpectedly");

    assert_eq!(msg2.source.0, "broadcast_agent1");
    assert_eq!(msg2.subject, "announcement");
    assert_eq!(msg2.payload, broadcast_payload);
    assert!(msg2.target.is_none(), "Broadcast message should have no target");

    // Agent 3 should also receive it
    let msg3 = tokio::time::timeout(Duration::from_secs(10), inbox3.next())
        .await
        .expect("agent3 timed out waiting for broadcast")
        .expect("agent3 stream ended unexpectedly");

    assert_eq!(msg3.source.0, "broadcast_agent1");
    assert_eq!(msg3.subject, "announcement");
    assert_eq!(msg3.payload, broadcast_payload);
    assert!(msg3.target.is_none());

    println!("✓ Broadcast message test passed");
}

#[tokio::test]
async fn test_offset_management() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();

    let client = create_test_client();
    let stream_id = Identifier::from_str_value("test_offset_stream").unwrap();

    ensure_test_stream(&client, &stream_id)
        .await
        .expect("Failed to ensure test stream");

    let transport1 = IggyTransport::new(
        client.clone(),
        stream_id.clone(),
        "test_offset",
    )
    .await
    .expect("Failed to create transport 1");

    let transport2 = IggyTransport::new(
        client.clone(),
        stream_id.clone(),
        "test_offset",
    )
    .await
    .expect("Failed to create transport 2");

    let sender = SimianAgent::new(AgentId::new("offset_sender"), transport1);
    let receiver = SimianAgent::new(AgentId::new("offset_receiver"), transport2);

    // Receiver subscribes first
    let mut inbox = receiver
        .listen()
        .await
        .expect("offset_receiver failed to subscribe");

    tokio::time::sleep(Duration::from_millis(500)).await;

    // Send multiple messages
    let num_messages = 3;
    let mut message_ids = Vec::new();

    for i in 0..num_messages {
        let payload = Bytes::from(format!("Message {}", i));
        let msg_id = sender
            .request(
                AgentId::new("offset_receiver"),
                "numbered_msg",
                payload,
            )
            .await
            .expect(&format!("Failed to send message {}", i));
        message_ids.push(msg_id);
        // Small delay between messages
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Receive and verify messages maintain order
    for i in 0..num_messages {
        let received = tokio::time::timeout(Duration::from_secs(10), inbox.next())
            .await
            .expect(&format!("Timeout waiting for message {}", i))
            .expect(&format!("Stream ended before message {}", i));

        assert_eq!(
            received.payload,
            Bytes::from(format!("Message {}", i)),
            "Message {} should have correct payload",
            i
        );
        assert_eq!(
            received.conversation_id, message_ids[i],
            "Message {} should have correct conversation ID",
            i
        );
    }

    println!("✓ Offset management and message ordering test passed");
}

#[tokio::test]
async fn test_connection_status() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();

    let client = create_test_client();
    let stream_id = Identifier::from_str_value("test_connection_stream").unwrap();

    ensure_test_stream(&client, &stream_id)
        .await
        .expect("Failed to ensure test stream");

    let transport = IggyTransport::new(
        client,
        stream_id,
        "test_connection",
    )
    .await
    .expect("Failed to create transport");

    // Verify transport reports as connected
    assert!(
        transport.is_connected().await,
        "Transport should report as connected"
    );

    println!("✓ Connection status test passed");
}

#[tokio::test]
async fn test_request_reply_pattern() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();

    let client = create_test_client();
    let stream_id = Identifier::from_str_value("test_reply_stream").unwrap();

    ensure_test_stream(&client, &stream_id)
        .await
        .expect("Failed to ensure test stream");

    let transport1 = IggyTransport::new(
        client.clone(),
        stream_id.clone(),
        "test_reply",
    )
    .await
    .expect("Failed to create transport 1");

    let transport2 = IggyTransport::new(
        client.clone(),
        stream_id.clone(),
        "test_reply",
    )
    .await
    .expect("Failed to create transport 2");

    let requester = SimianAgent::new(AgentId::new("requester"), transport1);
    let responder = SimianAgent::new(AgentId::new("responder"), transport2);

    // Responder listens first
    let mut responder_inbox = responder
        .listen()
        .await
        .expect("responder failed to subscribe");

    tokio::time::sleep(Duration::from_millis(500)).await;

    // Requester sends a request
    let request_payload = Bytes::from("What is the answer?");
    let conv_id = requester
        .request(
            AgentId::new("responder"),
            "question",
            request_payload.clone(),
        )
        .await
        .expect("requester failed to send request");

    // Responder receives the request
    let request_msg = tokio::time::timeout(Duration::from_secs(10), responder_inbox.next())
        .await
        .expect("responder timed out")
        .expect("responder stream ended");

    assert_eq!(request_msg.source.0, "requester");
    assert_eq!(request_msg.subject, "question");
    assert_eq!(request_msg.payload, request_payload);
    assert_eq!(request_msg.performative, Performative::Request);

    // Requester subscribes to receive the reply
    let mut requester_inbox = requester
        .listen()
        .await
        .expect("requester failed to subscribe for reply");

    tokio::time::sleep(Duration::from_millis(500)).await;

    // Responder sends a reply
    let response_payload = Bytes::from("42");
    responder
        .reply(&request_msg, Performative::Inform, response_payload.clone())
        .await
        .expect("responder failed to reply");

    // Requester receives the reply
    let reply_msg = tokio::time::timeout(Duration::from_secs(10), requester_inbox.next())
        .await
        .expect("requester timed out waiting for reply")
        .expect("requester stream ended");

    assert_eq!(reply_msg.source.0, "responder");
    assert_eq!(reply_msg.subject, "question.reply");
    assert_eq!(reply_msg.payload, response_payload);
    assert_eq!(reply_msg.performative, Performative::Inform);
    assert_eq!(reply_msg.conversation_id, conv_id);

    println!("✓ Request-reply pattern test passed");
}
