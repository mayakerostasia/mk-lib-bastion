# IggyTransport vs NATS: Key Differences

This document outlines the significant architectural and operational differences between using IggyTransport (Iggy) and the NATS-based transport for agent communication.

## Architecture Overview

### NATS Transport
- **Type**: Push-based pub/sub messaging
- **Persistence**: In-memory (optional JetStream for persistence)
- **Hierarchy**: Subject-based routing with wildcard support
- **State**: Stateless - no message offset tracking by broker
- **Consumer Model**: Server-initiated message delivery

### IggyTransport (Iggy)
- **Type**: Pull-based persistent streaming
- **Persistence**: Always persistent (disk-based)
- **Hierarchy**: Stream → Topic → Partition hierarchy
- **State**: Stateful - offset tracking per consumer
- **Consumer Model**: Client-initiated polling for messages

## Practical Differences

### 1. Stream and Topic Setup

**NATS**:
```rust
// Topics are implicit - they exist as long as there's a subscription
let monkey = Monkey::new("time.time", NATS_ADDR).await;
// Subject "time.time" is created on first use
```

**Iggy**:
```rust
// Must explicitly create stream first
let stream_id = Identifier::from_str_value("agents_stream")?;
client.create_stream(&stream_id, None).await?;

// IggyTransport auto-creates topics, but stream must exist
let transport = IggyTransport::new(client, stream_id, "agents").await?;
// Creates topics:
//   - agents_alice_inbox
//   - agents_bob_inbox
//   - agents_broadcast
```

**Key Difference**: Iggy requires stream pre-creation, while NATS topics are created implicitly.

### 2. Message Publishing

**NATS**:
```rust
// Synchronous publish to subject
let frame = Frame::message("hello");
let resp = monkey.msg(frame.into()).await?;
// Subscribing happens independently; publisher doesn't know about subscribers
```

**Iggy**:
```rust
// Publish to topic (auto-creates if needed)
let msg = AcpMessage { /* ... */ };
transport.publish(msg).await?;
// Topic is created automatically if it doesn't exist
// Messages are persisted immediately
```

**Key Difference**: NATS publishes to ephemeral subjects; Iggy publishes to persistent topics with automatic creation.

### 3. Message Consumption (Polling vs Subscription)

**NATS** (Push-based):
```rust
// Subscribe returns a stream of incoming messages
let mut subscription = client.subscribe("agents.alice").await?;
while let Some(msg) = subscription.next().await {
    // Process immediately as messages arrive
}
```

**Iggy** (Pull-based):
```rust
// Subscribe returns a polling stream
let mut stream = transport.subscribe(&alice_id).await?;

// Behind the scenes, polling happens at configured interval:
// - Polls agents_alice_inbox every poll_interval_ms
// - Polls agents_broadcast every poll_interval_ms
// - Tracks offset for each topic independently

while let Some(msg) = stream.next().await {
    // Messages come as they're polled
}
```

**Key Difference**: NATS pushes messages to subscribers; Iggy requires subscribers to pull messages.

### 4. Topic Naming Conventions

**NATS**:
- Uses dot-separated subjects: `agents.alice`, `agents.bob`, `agents.broadcast`
- Supports wildcards: `agents.*` matches all agent topics
- Hierarchical by convention, not enforced by broker

**Iggy**:
- Uses snake_case identifiers in topic names
- Format: `{base_topic}_{agent_id}_inbox` for individual agents
- Format: `{base_topic}_broadcast` for broadcasts
- Example with base_topic="agents":
  - `agents_alice_inbox` (messages for Alice)
  - `agents_bob_inbox` (messages for Bob)
  - `agents_broadcast` (messages for all agents)

**Key Difference**: NATS uses flexible subject naming; Iggy uses structured topic naming with agent IDs.

### 5. Partition and Load Balancing

**NATS**:
- No explicit partitioning concept
- Load balancing via consumer groups (JetStream)
- Round-robin delivery to group members

**Iggy**:
```rust
// Each topic has configurable partitions
let config = IggyTransportConfig {
    partitions: 4,  // Create topics with 4 partitions
    // ...
};

// Messages can be distributed across partitions
// Current implementation: Partition 1 for all messages
// Could be enhanced: Hash agent_id for distribution
let partitioning = Partitioning::partition_id(1);
```

**Key Difference**: Iggy explicit partitioning; NATS implicit via groups.

### 6. Consumer Groups

**NATS** (JetStream):
```rust
// Consumer group for load balancing
let durable = Some("my-consumer-group");
// Multiple subscribers in same group share messages
```

**Iggy**:
```rust
let config = IggyTransportConfig {
    consumer_group: Some("my_group".to_string()),
    // ...
};
// Consumer group coordinates offset tracking across members
```

**Key Difference**: Both support consumer groups, but semantics differ slightly.

### 7. Message Persistence

**NATS** (without JetStream):
- Messages lost on broker restart
- No replay capability

**NATS** (with JetStream):
- Messages persisted with configurable retention
- Can replay from stored offset

**Iggy**:
- All messages persisted to disk by default
- Automatic message replay from any offset
- Can rebuild consumer state from persistent log

**Key Difference**: Iggy always persistent; NATS requires JetStream for persistence.

### 8. Polling Configuration

**NATS**: No polling needed - server pushes messages

**Iggy**:
```rust
let config = IggyTransportConfig {
    poll_interval_ms: 100,    // Poll every 100ms
    batch_size: 10,           // Fetch up to 10 messages per poll
    partitions: 1,            // Partitions per topic
    consumer_group: None,     // Optional consumer group
};

// Polling behavior:
// - Regularly checks both inbox and broadcast topics
// - Fetches batch_size messages per topic per poll
// - Tracks offset independently per topic
// - Auto-commits offset after processing
```

**Key Difference**: Iggy requires tuning polling parameters; NATS has no polling overhead.

### 9. Offset Management

**NATS**:
- No explicit offset tracking (unless using JetStream consumer)
- Broker state independent of consumer state

**Iggy**:
```rust
// Offsets tracked per topic per consumer
let inbox_offset = 0u64;      // Start from message 0
let broadcast_offset = 0u64;

// After polling:
next_inbox = msg.header.offset + 1;      // Track offset
next_broadcast = msg.header.offset + 1;

// With auto_commit=true, offsets are persisted
// Can resume from last offset on restart
```

**Key Difference**: Iggy stateful with offset tracking; NATS stateless.

### 10. Error Handling and Retry

**NATS**:
```rust
// Failed delivery doesn't block subscriber
// Can configure redelivery policies in JetStream
let frame = Frame::error("failed");  // Explicit error frame
```

**Iggy**:
```rust
// Deserialization errors logged, stream continues
if let Ok(acp) = serde_json::from_slice::<AcpMessage>(&msg.payload) {
    combined.push(acp);
} else {
    tracing::error!("Failed to deserialize: {}", e);
    // Message skipped, offset still advances
}

// No automatic retry - handle at application level
```

**Key Difference**: NATS has built-in retry policies; Iggy requires explicit error handling.

## Migration Path from NATS

### 1. Update Dependencies
```toml
# Before
simian-nats-streams = { path = "../simian-nats-streams" }

# After
simian-iggy-streams = { path = "../simian-iggy-streams" }
```

### 2. Update Initialization
```rust
// Before: NATS
let nats_addr = "nats://localhost:4222";
let client = async_nats::connect(nats_addr).await?;
let transport = NatsTransport::new(client, "agents").await?;

// After: Iggy
let iggy_addr = "http://localhost:8090";
let client = IggyClient::new()
    .tcp(&iggy_addr)
    .and_then(|c| c.connect())
    .await?;
let stream_id = Identifier::from_str_value("agents_stream")?;
let transport = IggyTransport::new(client, stream_id, "agents").await?;
```

### 3. Topic Naming Updates
```rust
// Before: NATS subject-based
"agents.alice" → "agents.bob"

// After: Iggy topic-based
"agents_alice_inbox" ← "agents_bob_inbox"
```

### 4. Handling Polling
```rust
// NATS: Automatic push via server

// Iggy: Configure polling behavior
let config = IggyTransportConfig {
    poll_interval_ms: 100,  // Adjust based on latency needs
    batch_size: 10,         // Adjust based on throughput needs
    partitions: 1,
    consumer_group: None,
};
```

## Performance Considerations

### NATS Advantages
- Lower latency (push-based, no polling overhead)
- Simpler for request-response patterns
- Less memory overhead per topic
- Natural support for wildcards and patterns

### Iggy Advantages
- Built-in message replay from offset
- Disk persistence without additional configuration
- Natural ordering guarantees per partition
- Better for high-volume message processing
- Can consume messages at different rates
- Audit trail of all messages

## When to Use Each

### Use NATS When
- Low latency is critical
- Fire-and-forget messaging is acceptable
- Ephemeral topics (created/destroyed frequently)
- Complex subject hierarchies needed
- Server-side routing preferred

### Use Iggy When
- Message persistence is required
- Replay capability is needed
- Load balancing across multiple consumers needed
- Audit trail needed
- High throughput steady-state processing
- Can tolerate polling latency (~100ms typical)

## Configuration Reference

### IggyTransportConfig Fields

```rust
pub struct IggyTransportConfig {
    /// Time between polls (ms)
    pub poll_interval_ms: u64,        // Default: 100
    
    /// Messages per poll
    pub batch_size: u32,              // Default: 10
    
    /// Partitions created for new topics
    pub partitions: u32,              // Default: 1
    
    /// Consumer group for coordination
    pub consumer_group: Option<String>, // Default: None
}
```

### Recommended Configurations

**Low Latency (minimize polling overhead)**:
- `poll_interval_ms: 50`
- `batch_size: 5`

**High Throughput**:
- `poll_interval_ms: 100`
- `batch_size: 50`
- `partitions: 4` (or more for heavy load)

**Balanced**:
- `poll_interval_ms: 100`
- `batch_size: 10`
- `partitions: 1`

## Debugging and Monitoring

### NATS
```rust
// Subscribe to debug topics
client.subscribe("$SYS.>").await?;
```

### Iggy
```rust
// Check stream/topic status
client.get_stream(&stream_id).await?;
client.get_topic(&stream_id, &topic_id).await?;

// View partition details
// (Requires Iggy CLI or admin tools)
```

## Examples Provided

### 1. `frame_serialization.rs`
- Demonstrates message serialization with IggyTransport
- Shows automatic topic creation
- Compares ACP message structure with NATS Frame
- Documents topic naming conventions

### 2. `simple_agent.rs`
- Shows two agents communicating
- Demonstrates direct messaging and broadcasting
- Shows polling-based message reception
- Configures custom polling parameters

## Testing IggyTransport

```bash
# Start Iggy server
docker run -it --rm -p 3000:3000 -p 8090:8090 iggydata/iggy:latest

# Run frame_serialization example
IGGY_ADDR=http://localhost:8090 cargo run --example frame_serialization

# Run simple_agent example
IGGY_ADDR=http://localhost:8090 cargo run --example simple_agent
```

## Troubleshooting

### "Topic must be created before use"
- Ensure stream exists: `client.create_stream(&stream_id, None).await?`
- IggyTransport creates topics automatically; stream is required

### "Timeout waiting for messages"
- Increase `poll_interval_ms` if server is slow
- Verify Iggy server is running: `curl http://localhost:8090/stats`
- Check network connectivity to Iggy server

### Messages not being delivered
- Verify agent ID matches topic naming: `base_topic_agent_id_inbox`
- Check that publisher is waiting before subscriber polls (timing issue)
- Ensure stream has been created

### High memory usage
- Reduce `batch_size` if fetching too many messages
- Reduce number of topics (combine multiple agents if possible)
- Adjust retention policies in Iggy server
