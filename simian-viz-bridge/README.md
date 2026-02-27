# simian-viz-bridge

WebSocket bridge that connects NATS agent messages to browser visualization using **proper Transport abstraction**.

## Architecture

```
Browser (React) ←─ WebSocket ─← viz-bridge (as Agent) ←─ Transport ←─ NATS ←─ Agents
```

**Key Design**: viz-bridge subscribes as a special observer agent (`__viz_observer__`) using the `Transport` trait, receiving all agent messages through the proper ACP protocol.

## Why This Matters

✅ **Type-safe**: Uses `AcpMessage` instead of manual JSON parsing  
✅ **Consistent**: Same transport layer as all other agents  
✅ **Maintainable**: Changes to ACP protocol automatically propagate  
✅ **Extensible**: Can filter by performative, query agents, etc.  
✅ **Testable**: Can mock Transport for testing

## Usage

```bash
# Run locally
NATS_URL=nats://localhost:4222 WS_PORT=3030 cargo run

# Or with Docker
docker build -t simian-viz-bridge -f docker/Dockerfile.viz-bridge .
docker run -e NATS_URL=nats://nats:4222 -p 3030:3030 simian-viz-bridge
```

## How It Works

1. **Creates NatsTransport** - Connects to NATS using our Transport abstraction
2. **Subscribes as Observer** - Special agent ID `__viz_observer__` receives broadcasts
3. **Transforms Messages** - Converts `AcpMessage` → `VizMessage` (browser-friendly JSON)
4. **Broadcasts via WebSocket** - Sends to all connected browser clients

## WebSocket Protocol

Connect to `ws://localhost:3030/ws` and receive JSON messages:

```json
{
  "timestamp": 1234567890,
  "from_agent": "generator-001",
  "to_agent": "processor-001",
  "performative": "Request",
  "message_id": "550e8400-e29b-41d4-a716-446655440000",
  "conversation_id": "550e8400-e29b-41d4-a716-446655440001"
}
```

## API Endpoints

- `GET /ws` - WebSocket upgrade endpoint
- `GET /health` - Health check (returns "OK")

## Environment Variables

- `NATS_URL` - NATS server URL (default: `nats://localhost:4222`)
- `BASE_SUBJECT` - NATS base subject (default: `agents`)
- `WS_PORT` - WebSocket server port (default: `3030`)
- `RUST_LOG` - Log level (default: `info`)

## Benefits Over Raw NATS

**Before** (naive implementation):
```rust
// Manually subscribe to NATS subjects
nats_client.subscribe("agents.>").await?
// Manually parse JSON
let acp_msg: AcpMessage = serde_json::from_slice(&payload)?
```

**After** (proper Transport abstraction):
```rust
// Use Transport trait
let transport = NatsTransport::new(&nats_url, "agents").await?;
// Get typed stream
let mut stream = transport.subscribe(&AgentId("__viz_observer__")).await?;
while let Some(acp_msg) = stream.next().await {
    // acp_msg is already an AcpMessage!
}
```

## Advanced Features (Possible)

Since we're using the Transport trait, we can easily add:

- **Filtering**: `acp_msg.performative == Performative::Request`
- **Querying**: Send Request messages to agents, get responses
- **Conversation tracking**: Follow `conversation_id` chains
- **Agent discovery**: Query who's online via Request/Inform
- **Replay**: Store messages and replay for debugging

## Testing

```rust
#[tokio::test]
async fn test_transform() {
    let acp = AcpMessage { /* ... */ };
    let viz = transform_acp_to_viz(&acp).unwrap();
    assert_eq!(viz.from_agent, "test-agent");
}
```
