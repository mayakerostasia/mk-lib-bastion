# IggyTransport Examples

This directory contains working examples demonstrating how to use IggyTransport for agent communication.

## Quick Start

### Prerequisites
1. Iggy server must be running:
```bash
docker run -it --rm -p 3000:3000 -p 8090:8090 iggydata/iggy:latest
```

2. Optionally, set the Iggy address (defaults to `http://localhost:8090`):
```bash
export IGGY_ADDR=http://localhost:8090
```

## Examples

### 1. frame_serialization.rs
**Purpose**: Demonstrates message serialization and the differences from NATS Frame-based communication.

**What it shows**:
- Creating ACP messages (Agent Communication Protocol)
- Publishing messages through IggyTransport
- Automatic topic creation based on agent IDs
- Receiving broadcast and direct messages
- Comparing message formats with NATS version

**Run it**:
```bash
cargo run --example frame_serialization
```

**Key Learning Points**:
- Topic naming: `{base_topic}_{agent_id}_inbox` for direct messages
- Topic naming: `{base_topic}_broadcast` for broadcast messages
- AcpMessage vs NATS Frame differences
- Pull-based polling vs NATS push-based delivery

### 2. simple_agent.rs
**Purpose**: Shows complete agent-to-agent communication patterns.

**What it shows**:
- Creating multiple agents
- Sending direct messages between agents
- Broadcasting messages to all agents
- Receiving messages with configurable polling
- Handling message streams with timeouts
- Configuring IggyTransportConfig for custom polling behavior

**Run it**:
```bash
cargo run --example simple_agent
```

**Key Learning Points**:
- Agent creation and initialization
- Message flow between agents
- Broadcasting vs direct messaging
- Polling configuration trade-offs
- Timeout handling for demo purposes

## Architecture Overview

### Stream/Topic Hierarchy
```
agents_stream (Stream)
├── agents_alice_inbox (Topic - messages for Alice)
├── agents_bob_inbox (Topic - messages for Bob)
└── agents_broadcast (Topic - messages for all agents)
```

### Message Flow
```
Alice                           Bob
  │                             │
  ├─────(direct message)───────→│
  │                             │
  ├────(broadcast message)──────→│
  │                             │
  │←────(direct message)─────────┤
  │                             │
  ├←────(broadcast message)──────┤
  │                             │
```

Each topic uses polling to pull messages:
- Poll interval: 100ms (configurable)
- Batch size: 10 messages (configurable)
- Offsets tracked independently per topic
- Messages persisted to disk

## Configuration

### IggyTransportConfig
```rust
pub struct IggyTransportConfig {
    /// Time to wait between polling attempts (milliseconds)
    pub poll_interval_ms: u64,
    /// Number of messages to fetch per poll
    pub batch_size: u32,
    /// Number of partitions per topic
    pub partitions: u32,
    /// Optional consumer group for load balancing
    pub consumer_group: Option<String>,
}
```

### Default Configuration
```rust
IggyTransportConfig::default()
// poll_interval_ms: 100
// batch_size: 10
// partitions: 1
// consumer_group: None
```

### Custom Configuration Example
```rust
let config = IggyTransportConfig {
    poll_interval_ms: 250,              // Poll every 250ms
    batch_size: 50,                     // Fetch up to 50 messages
    partitions: 4,                      // Create topics with 4 partitions
    consumer_group: Some("group1".into()), // Join consumer group
};

let transport = IggyTransport::new_with_config(
    client,
    stream_id,
    "agents",
    config
).await?;
```

## Common Patterns

### Creating an Agent
```rust
// 1. Create Iggy client
let client = IggyClient::new()
    .tcp("http://localhost:8090")
    .and_then(|c| c.connect())
    .await?;

// 2. Ensure stream exists
let stream_id = Identifier::from_str_value("agents_stream")?;
client.create_stream(&stream_id, None).await?;

// 3. Create transport
let transport = IggyTransport::new(client, stream_id, "agents").await?;

// 4. Use in agent
let agent_id = AgentId::new("alice");
```

### Sending Messages
```rust
// Direct message to another agent
let msg = AcpMessage {
    message_id: Uuid::new_v4(),
    source: AgentId::new("alice"),
    target: Some(AgentId::new("bob")),  // Specific target
    performative: Performative::Request,
    subject: "greeting".to_string(),
    conversation_id: Uuid::new_v4(),
    payload: Bytes::from("Hello Bob!"),
    timestamp: chrono::Local::now().timestamp_millis(),
};

transport.publish(msg).await?;
```

### Broadcasting Messages
```rust
// Broadcast to all agents
let msg = AcpMessage {
    message_id: Uuid::new_v4(),
    source: AgentId::new("alice"),
    target: None,  // No target = broadcast
    performative: Performative::Inform,
    subject: "status".to_string(),
    conversation_id: Uuid::new_v4(),
    payload: Bytes::from("I'm online"),
    timestamp: chrono::Local::now().timestamp_millis(),
};

transport.publish(msg).await?;
```

### Receiving Messages
```rust
// Subscribe to messages (direct + broadcast)
let mut stream = transport.subscribe(&AgentId::new("alice")).await?;

// Poll for messages with timeout
let timeout = tokio::time::timeout(
    Duration::from_secs(5),
    async {
        while let Some(msg) = stream.next().await {
            println!("Received: {:?}", msg);
        }
    }
).await;
```

## Key Differences from NATS

### Delivery Model
- **NATS**: Push-based (server sends messages to subscribers)
- **Iggy**: Pull-based (clients poll for messages)

### Persistence
- **NATS**: Optional (requires JetStream)
- **Iggy**: Always persistent to disk

### Topic Setup
- **NATS**: Implicit (created on first use)
- **Iggy**: Stream must be created, topics auto-created

### Naming
- **NATS**: Subject-based with wildcards (`agents.alice`)
- **Iggy**: Topic-based with agent IDs (`agents_alice_inbox`)

### Polling
- **NATS**: No polling (push model)
- **Iggy**: Configurable polling (default 100ms)

### Offset Tracking
- **NATS**: No offset tracking (stateless)
- **Iggy**: Offset tracking per topic per consumer

See `NATS_VS_IGGY.md` for detailed comparison.

## Environment Variables

```bash
# Set Iggy server address (default: http://localhost:8090)
export IGGY_ADDR=http://your-iggy-server:8090
```

## Troubleshooting

### Connection Issues
```
Error: "Failed to connect to Iggy server"
Solution: Verify Iggy is running and IGGY_ADDR is correct
```

```bash
# Check if Iggy is running
curl http://localhost:8090/stats
```

### Topic Not Found
```
Error: "Topic doesn't exist"
Solution: Verify stream is created before creating transport
```

### No Messages Received
```
Symptoms: Listener times out without receiving messages
Causes:
1. Publisher hasn't sent messages yet
2. Polling interval too long (increase frequency)
3. Wrong agent ID in subscription
4. Network connectivity issue
```

### Memory Usage High
```
Solutions:
1. Reduce batch_size in config
2. Increase poll_interval_ms (trade latency for fewer polls)
3. Check Iggy server isn't retaining too many messages
```

## Performance Tuning

### For Low Latency
```rust
IggyTransportConfig {
    poll_interval_ms: 50,   // Poll more frequently
    batch_size: 5,          // Small batches
    partitions: 1,
    consumer_group: None,
}
```

### For High Throughput
```rust
IggyTransportConfig {
    poll_interval_ms: 100,  // Standard polling
    batch_size: 100,        // Large batches
    partitions: 4,          // Multiple partitions
    consumer_group: Some("group".into()), // Load balancing
}
```

### For Balanced Performance
```rust
IggyTransportConfig::default()
// poll_interval_ms: 100
// batch_size: 10
// partitions: 1
// consumer_group: None
```

## Testing

### Run with Default Settings
```bash
cargo run --example frame_serialization
cargo run --example simple_agent
```

### Run with Custom Iggy Address
```bash
IGGY_ADDR=http://remote-iggy:8090 cargo run --example simple_agent
```

### Run Tests (if any)
```bash
cargo test --lib
```

## Additional Resources

- `NATS_VS_IGGY.md` - Detailed comparison of NATS and Iggy approaches
- `../README.md` - Main simian-iggy-streams documentation
- Iggy Documentation: https://iggy.rs/
- ACP (Agent Communication Protocol): Defined in simian-base-api

## Example Output

### frame_serialization.rs Output
```
Starting Frame Serialization Example with IggyTransport

Connecting to Iggy at: http://localhost:8090
Connected to Iggy!

Created IggyTransport with base_topic='agents'

Message 1 (Alice -> Bob):
  Type: Request
  Subject: greeting
  Payload: Hello Bob!
  Publishing to: agents_bob_inbox

[... messages published ...]

All messages published successfully!

Subscribing to messages for Alice...

Alice received message 1:
  From: bob
  Performative: Inform
  Subject: status
  Payload: I'm online and ready

[... more messages ...]
```

### simple_agent.rs Output
```
=== Simple Agent Example with IggyTransport ===

Connecting to Iggy at: http://localhost:8090

✓ Connected to Iggy
✓ Using existing stream: agents_stream
✓ Created IggyTransport

Creating agent: alice
Creating agent: bob

--- Message Flow ---

[alice] Sending message to bob: 'hello' - Hi Bob, how are you?
[bob] Sending message to alice: 'reply' - I'm doing great!
[bob] Broadcasting message: 'status_update' - All systems operational
[alice] Broadcasting message: 'greeting' - Hello everyone!

--- Messages Received ---

[alice] Received 2 messages:
  1. From 'bob': 'reply' - I'm doing great!
  2. From 'bob': 'status_update' - All systems operational

[bob] Received 2 messages:
  1. From 'alice': 'hello' - Hi Bob, how are you?
  2. From 'alice': 'greeting' - Hello everyone!
```
