# Migration Guide - simian-bastion v2.1.0

This guide covers breaking changes and migration paths for updating to simian-bastion v2.1.0.

## Table of Contents

1. [Configuration System Changes](#configuration-system-changes)
2. [Event System Enhancements](#event-system-enhancements)
3. [HTTP Listener Changes](#http-listener-changes)
4. [Iggy Transport](#iggy-transport)
5. [Frame Protocol](#frame-protocol)

---

## Configuration System Changes

### simian-bin-cfg-creator → simian-config

**Status**: ✅ **COMPLETED**

The standalone `simian-bin-cfg-creator` binary has been merged into `simian-config`.

#### Before (v2.0.x)
```bash
# Old way - separate binary crate
cargo run --bin cfg-gen -- surreal

# Old API
use simian_config::srql_config;
let cfg = srql_config()?;
```

#### After (v2.1.0)
```bash
# New way - integrated binary
cargo run -p simian-config --bin cfg-gen -- surreal

# New API (recommended)
use simian_config::{ConfigLoader, SurrealCfg};
let cfg = ConfigLoader::new("SURREAL")
    .from_env()
    .from_file("config.json")
    .load::<SurrealCfg>()?;

# Old API still works (backward compatible)
use simian_config::srql_config;
let cfg = srql_config()?;  // Still supported!
```

#### Migration Steps

1. **Update binary commands**: Change `--bin cfg-gen` to `-p simian-config --bin cfg-gen`
2. **Optional**: Migrate to new ConfigLoader API for better flexibility
3. **No code changes required** if using old API - fully backward compatible

---

## Event System Enhancements

### New Event Observability System

**Status**: ✅ **COMPLETED**

A new comprehensive event capture system has been added alongside the existing lightweight emitter.

#### Before (v2.0.x)
```rust
use simian_event::{Emitter, SimianEvent, SimianEventType};

let emitter = Emitter::new();
emitter.emit_event("my-event", frame, "nats://localhost:4222").await?;
```

#### After (v2.1.0)
```rust
// Old API still works
use simian_event::{Emitter, SimianEvent, SimianEventType};
let emitter = Emitter::new();
emitter.emit_event("my-event", frame, "nats://localhost:4222").await?;

// New enhanced API (optional)
use simian_event::{EventCapture, MemoryEventSink, SeverityFilter, EventSeverity, EventCategory, EventData};
use std::sync::Arc;

let sink = Arc::new(MemoryEventSink::new(1000));
let capture = EventCapture::new("my-service")
    .with_sink(sink)
    .with_filter(Arc::new(SeverityFilter::new(EventSeverity::Info)));

capture.capture(
    EventCategory::Service,
    EventSeverity::Info,
    EventData::ServiceRequest {
        service: "my-service".to_string(),
        operation: "process".to_string(),
        request_id: "req-123".to_string(),
    },
).await?;
```

#### New Features
- **Pluggable sinks**: `MemoryEventSink`, `FileEventSink`, or custom implementations
- **Composable filters**: Filter by severity, category, source, or custom logic
- **Rich event data**: 12 event data variants covering all system operations
- **Thread-safe**: Use with `Arc` for sharing across tasks

#### Migration Steps
**No migration required** - the old `Emitter` API is unchanged. Use the new `EventCapture` API for advanced use cases.

---

## HTTP Listener Changes

### Health Checks and Messaging Endpoints

**Status**: ✅ **COMPLETED**

New features added to `simian-http-listener`:

#### Health Checks (v2.1.0)
```rust
use simian_http_listener::health::{HealthChecker, MemoryHealthCheck, UptimeHealthCheck};
use std::sync::Arc;

let checker = Arc::new(
    HealthChecker::new()
        .with_check(Arc::new(MemoryHealthCheck::new(1024 * 1024 * 100))) // 100MB threshold
        .with_check(Arc::new(UptimeHealthCheck::new()))
);

// Add to your Axum router
let app = Router::new()
    .route("/healthz", get(health_handler))
    .with_state(checker);
```

#### HTTP Messaging Endpoint (v2.1.0)
```rust
use simian_http_listener::handlers::messaging::{send_message_handler, MessageSender};

// Implement MessageSender trait for your transport
struct MyMessageSender { /* ... */ }

#[async_trait]
impl MessageSender for MyMessageSender {
    async fn send(&self, frame: Frame) -> Result<()> {
        // Send frame via your transport
        Ok(())
    }
}

// Add to router
let sender = Arc::new(MyMessageSender::new());
let app = Router::new()
    .route("/message", post(send_message_handler))
    .with_state(sender);
```

#### Migration Steps
**No migration required** - these are additive features. Add them to your router if needed.

---

## Iggy Transport

### Iggy Transport Now Stable

**Status**: ✅ **COMPLETED**

`simian-iggy-streams` has been fixed and re-enabled in the workspace.

#### Usage
```rust
use simian_iggy_streams::IggyTransport;
use simian_base_api::transport::Transport;

let transport = IggyTransport::new(
    "localhost:8090",
    "my-stream",
    "my-topic",
).await?;

// Use like any other Transport
transport.publish(message).await?;
let messages = transport.subscribe(&agent_id).await?;
```

#### Key Differences from NATS
- **Pull-based**: Uses polling instead of push notifications
- **Persistent streams**: Messages are stored and can be replayed
- **Configurable**: polling interval, batch size, consumer groups

See `simian-iggy-streams/examples/NATS_VS_IGGY.md` for detailed comparison.

---

## Frame Protocol

### Frame Serialization with rkyv

**Status**: ✅ **COMPLETED**

Frame protocol now supports zero-copy serialization via rkyv.

#### Usage
```rust
use simian_reactor::protocol::Frame;

// Encode to bytes
let frame = Frame::Msg("hello".to_string());
let bytes = frame.encode()?;

// Decode from bytes
let decoded = Frame::decode(&bytes)?;
```

#### Performance
- ~1.3µs per encode/decode cycle
- Zero-copy deserialization
- Handles all Frame variants including nested `SendBox`

#### Migration Steps
**No migration required** - this is a new capability. Existing serialization methods still work.

---

## Breaking Changes Summary

### None!

**All changes in v2.1.0 are backward compatible**:
- ✅ Old config API still works
- ✅ Old event emitter unchanged  
- ✅ No changes to core Transport trait
- ✅ No changes to Agent API
- ✅ Frame protocol is additive only

### Deprecated Features

- **simian-bin-cfg-creator** - The directory remains with a `DEPRECATED.md` file pointing to the new location
  - **Action**: Update scripts to use `cargo run -p simian-config --bin cfg-gen`
  - **Timeline**: Will be fully removed in v3.0.0

---

## Recommended Upgrade Path

1. **Update Cargo.toml**:
   ```toml
   simian-config = { path = "../simian-config", version = "2.1.0" }
   simian-event = { path = "../simian-event", version = "2.1.0" }
   # ... other crates
   ```

2. **Run tests**: `cargo test --workspace`

3. **Update scripts**: Replace `--bin cfg-gen` with `-p simian-config --bin cfg-gen`

4. **Optional**: Explore new features (EventCapture, health checks, Iggy transport)

---

## Getting Help

- **Architecture**: See `ARCHITECTURE.md` for system design
- **Examples**: Each crate has examples in its `examples/` directory
- **Issues**: File issues on the repository

---

## Future Plans (v3.0.0)

The following may include breaking changes in future releases:

- **simian-base-api refactoring**: Splitting protocol definitions into `simian-protocol` crate
- **Agent module restructuring**: Moving agent-specific code out of base-api
- **Feature flag simplification**: Consolidating configuration features

These are **not happening in v2.1.0** - this is future planning only.
