# Simian Agent Server

A flexible, long-lived agent server for processing NATS Frame messages using the Kong/Monkey pattern.

## Features

- 🚀 **Long-lived server** - Runs continuously, perfect for Docker/Kubernetes
- 🏥 **Built-in health checks** - HTTP endpoint for container orchestration
- 🎭 **Role-based behavior** - Switch agent functionality via environment variables
- 🏗️ **Tower middleware** - Optional buffering, rate limiting, timeouts, concurrency control
- 📊 **Observability** - OpenTelemetry tracing integration
- 🛑 **Graceful shutdown** - Handles SIGTERM/SIGINT cleanly

## Quick Start

```bash
# Build
cargo build -p simian-bin-agent

# Run with defaults (echo agent)
cargo run -p simian-bin-agent

# Run with specific role
AGENT_ROLE=processor cargo run -p simian-bin-agent

# Test with simian-bin-monkey
AGENT_ROLE=processor cargo run -p simian-bin-agent &
cargo run -p simian-bin-monkey -- --subject "agents.agent-001" --cmd "uppercase" --args "hello world"
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `NATS_URL` | `nats://localhost:4222` | NATS server address |
| `AGENT_ID` | `agent-001` | Unique agent identifier (becomes NATS subject) |
| `AGENT_ROLE` | `echo` | Agent role/behavior (see Roles below) |
| `AGENT_SUBJECT` | `agents` | Base NATS subject |
| `HEALTH_BIND` | `0.0.0.0:6660` | Health check HTTP endpoint |
| `RUST_LOG` | `info` | Log level (trace, debug, info, warn, error) |

## Agent Roles

### `echo` - Simple Echo Server
Returns received frames unchanged. Good for testing connectivity.

```bash
AGENT_ROLE=echo simian-agent
```

### `processor` - Message Processor
Transforms messages with processing metadata. Supports commands:
- `uppercase <text>` - Convert to uppercase
- `lowercase <text>` - Convert to lowercase
- `reverse <text>` - Reverse text
- `count <text>` - Count characters

```bash
AGENT_ROLE=processor simian-agent

# Test
simian-bin-monkey --subject "agents.agent-001" --cmd "uppercase" --args "hello"
# Response: "HELLO"
```

### `generator` - Data Generator
Generates data based on commands:
- `uuid` - Generate UUID
- `random [max]` - Generate random number
- `timestamp` - Current timestamp
- `sequence [count]` - Number sequence

```bash
AGENT_ROLE=generator simian-agent

# Test
simian-bin-monkey --subject "agents.agent-001" --cmd "uuid"
# Response: "550e8400-e29b-41d4-a716-446655440000"
```

### `verifier` - Data Validator
Validates messages and data structures:
- Message validation (empty, length)
- JSON validation (structure, type)
- Command validation

```bash
AGENT_ROLE=verifier simian-agent

# Test
simian-bin-monkey --subject "agents.agent-001" --cmd "test" --args "Hello"
# Response: {"status": "VALID", "command": "test", "arg_count": 1}
```

### `tower-example` - Full Middleware Stack
Demonstrates Tower service with:
- Buffer: 10 requests
- Concurrency limit: 5 concurrent
- Timeout: 30 seconds
- Rate limit: 100 req/second

```bash
AGENT_ROLE=tower-example simian-agent
```

## Docker Usage

### Build Image
```bash
docker build -f docker/Dockerfile --build-arg RUST_BIN=simian-bin-agent -t simian-agent:latest .
```

### Run Container
```bash
docker run -d \
  -e NATS_URL=nats://nats:4222 \
  -e AGENT_ID=processor-001 \
  -e AGENT_ROLE=processor \
  -e RUST_LOG=info \
  -p 6660:6660 \
  simian-agent:latest
```

### Health Check
```bash
curl http://localhost:6660/healthz
```

## Docker Compose Example

```yaml
services:
  agent-processor:
    build:
      context: .
      dockerfile: docker/Dockerfile
      args:
        RUST_BIN: simian-bin-agent
    environment:
      - NATS_URL=nats://nats:4222
      - AGENT_ID=processor-001
      - AGENT_ROLE=processor
      - RUST_LOG=info
    depends_on:
      nats:
        condition: service_healthy
    healthcheck:
      test: ["CMD", "wget", "--spider", "-q", "http://localhost:6660/healthz"]
      interval: 10s
      timeout: 5s
      retries: 3
```

## Architecture

### Kong/Monkey Pattern
- **Kong** = Server/Listener (this agent)
- **Monkey** = Client/Sender (simian-bin-monkey)

```
┌─────────────┐          ┌──────┐          ┌─────────────┐
│   Monkey    │─Frame─>  │ NATS │  ─Frame─>│    Kong     │
│  (Client)   │          └──────┘          │  (Server)   │
└─────────────┘                            │             │
                                           │ ┌─────────┐ │
                                           │ │ Handler │ │
                                           │ └─────────┘ │
                                           └─────────────┘
```

### NATS Subject Pattern
```
{base_subject}.{agent_id}
     ↓              ↓
  "agents"   "processor-001"
         ↓
"agents.processor-001"  ← Full subject Kong listens on
```

### Health Endpoint
```
GET /healthz  → HTTP 200 OK (if healthy)
```

## Advanced Usage

### Custom Handler
Create your own role by adding to `src/roles.rs`:

```rust
pub async fn custom_handler(frame: Frame) -> Result<Frame, BoxError> {
    // Your logic here
    Ok(frame)
}
```

Then register in `src/main.rs`:
```rust
"custom" => {
    kkong.new_future_kong(&args.agent_id, roles::custom_handler).await?;
}
```

### Tower Service with Custom Middleware
```rust
let service = ServiceBuilder::new()
    .buffer(20)
    .rate_limit(50, Duration::from_secs(1))
    .service(MyService);
    
kkong.new_tower_kong("handler", service).await?;
```

## Comparison with simian-bin-monkey

| Feature | simian-bin-monkey | simian-bin-agent |
|---------|------------------|------------------|
| Purpose | CLI tool (client) | Server (daemon) |
| Lifecycle | Send & exit | Run forever |
| Pattern | Monkey (sender) | Kong (listener) |
| Use case | Testing, one-off commands | Production agents |
| Docker | Not suitable | Perfect fit |

## Troubleshooting

### Agent not receiving messages
- Check NATS connection: `RUST_LOG=debug simian-agent`
- Verify subject matches: `agents.{AGENT_ID}`
- Test with monkey: `simian-bin-monkey --subject "agents.agent-001"`

### Health check failing
- Verify port: `curl http://localhost:6660/healthz`
- Check HEALTH_BIND env var
- Look for port conflicts

### High latency
- Enable Tower middleware: `AGENT_ROLE=tower-example`
- Adjust concurrency limit in roles.rs
- Check NATS server performance

## Examples

See the previous session examples for inspiration:
- `simian-nats-streams/examples/tower.rs` - Tower service
- `simian-nats-streams/examples/wtapi-server.rs` - Future-based
- `simian-event/examples/start_emitter.rs` - Simple closure

## License

Same as parent project.
