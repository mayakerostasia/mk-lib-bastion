# Copilot Instructions for simian-bastion

This is a Rust workspace containing libraries and binaries for building agent-based systems with pluggable transport backends.

## Build, Test, and Lint

```bash
# Build entire workspace
cargo build --workspace

# Build a specific package
cargo build -p simian-base-api

# Run tests for entire workspace
cargo test --workspace --verbose

# Run tests for a specific package
cargo test -p simian-nats-streams

# Run a specific test
cargo test -p simian-base-api test_name

# Format code
cargo fmt

# Check code without building
cargo check --workspace

# Run clippy (if used)
cargo clippy --workspace
```

## Architecture Overview

### Agent Communication Protocol (ACP)

The core abstraction is the **Transport trait** defined in `simian-base-api/src/transport.rs`, which enables pluggable messaging backends. All transport implementations must support:
- Publishing `AcpMessage` structs
- Subscribing to agent-specific message streams
- Connection state management

**AcpMessage Structure:**
- Uses UUID-based message IDs and conversation IDs
- Supports performatives (Request, Inform, Query, Agree, Refuse, Failure, Subscribe, Unsubscribe)
- Agents identified by `AgentId` (string-based)
- Payloads are `Bytes` for flexibility

### Transport Implementations

**NATS Transport** (`simian-nats-streams`):
- Subject pattern: `{base_subject}.{agent_id}.inbox` for direct messages
- Subject pattern: `{base_subject}.broadcast` for broadcasts
- Merges inbox and broadcast subscriptions into single stream
- Additional features: Frame-based messaging, Kong/Monkey utilities for NATS interaction

**Iggy Transport** (`simian-iggy-streams`):
- Alternative streaming backend implementation
- Follows same Transport trait contract

### High-Level Agent API

`SimianAgent<T: Transport>` in `simian-base-api/src/agent.rs` provides convenience methods:
- `request()` - Send request to specific agent
- `inform()` - Send informational message (can broadcast with `None` target)
- `reply()` - Reply to previous message (maintains conversation_id)
- `listen()` - Subscribe to messages for this agent

### Frame Protocol (NATS-specific)

The `simian-nats-streams` crate includes a Frame-based protocol for structured messaging:
- **Frame enum**: `Ping`, `Pong`, `Msg`, `Bytes`, `Json`, `Exec`, `Fin`, `Error`, `SendBox`
- **Encoder/Decoder traits**: Serialize/deserialize frames using `bincode`
- **Kong/Monkey pattern**: 
  - **KingKong**: NATS subscriber that processes incoming frames (server-side)
  - **Monkey**: NATS publisher that sends frames (client-side)
- **simian-bin-monkey**: CLI tool to send Frame messages to NATS subjects

### Supporting Libraries

**simian-reactor**: Tower-based service middleware with `MakoReactor`, `MakoLayer`, and `MakoBattery` for composable request/response processing.

**simian-event**: Event emitter/receiver system built on top of transport layer. Provides `Emitter` and `SimianEvent`/`SimianEventType` abstractions.

**simian-config**: Feature-gated configuration management. Enable features: `dev`, `surreal`, `autotask`, `jira`, `swimlane`.

**simian-tracing**: Observability with OpenTelemetry, Loki, and tracing integration.

**simian-metrics**: Prometheus metrics exporter with HTTP listener and push gateway support.

**simian-surreal-client**: SurrealDB client with Tower service integration and live query support.

**simian-http-listener**: Axum-based HTTP listener utilities.

## Key Conventions

### Transport Trait Implementation Pattern

When implementing a new Transport:
1. Struct should derive/implement `Clone` and `Debug`
2. Use `#[async_trait]` for the Transport trait implementation
3. Serialize/deserialize `AcpMessage` using `serde_json`
4. Log deserialization failures with `tracing::error!` rather than panicking
5. Return `BoxStream<'static, AcpMessage>` from subscribe methods

Example from `simian-nats-streams`:
```rust
#[derive(Clone)]
pub struct NatsTransport {
    client: Client,
    base_subject: String,
}

impl Debug for NatsTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NatsTransport")
            .field("base_subject", &self.base_subject)
            .finish()
    }
}
```

### Package Naming

- `simian-*` prefix for library crates (transport implementations, utilities)
- `simian-bin-*` prefix for binary crates (executables/tools)
- Crate names use underscores in imports: `simian_base_api`, `simian_nats_streams`
- Type names use SimianPrefix: `SimianAgent`, `SimianEvent`, `SimianFrame`

### Testing

- Use `#[cfg(test)]` modules within source files for unit tests
- Integration tests go in `tests/` directory
- Examples in `examples/` directory demonstrate real usage patterns
- Common pattern: comprehensive test coverage with `#[cfg(test)]` modules in most components

### Error Handling

- Use `anyhow::Result` for most public APIs
- Define custom error types with `thiserror::Error` for domain-specific errors
- Use `tracing::error!` for logging errors rather than printing to stderr

### Dependencies

All workspace dependencies are centralized in root `Cargo.toml` under `[workspace.dependencies]`. Use `workspace = true` in package-specific Cargo.toml files to reference them.

### Tower Service Pattern

Many components integrate with Tower's `Service` trait for middleware composition. When implementing Tower services:
- Use `Pin<Box<dyn Future>>` for the `Future` associated type
- Implement `poll_ready` to manage backpressure
- The reactor crate provides `MakoLayer` for wrapping services with common patterns

