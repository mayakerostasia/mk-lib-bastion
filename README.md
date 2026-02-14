# simian-bastion

A Rust workspace for building agent-based systems with pluggable transport backends, Tower-based middleware composition, and structured message framing.

---

## Table of Contents

- [Architecture Overview](#architecture-overview)
- [Crate Map](#crate-map)
- [simian-base-api](#simian-base-api) — Core ACP transport trait and `SimianAgent`
- [simian-nats-streams](#simian-nats-streams) — NATS transport, Frame protocol, Kong/Monkey, Tower services
- [simian-iggy-streams](#simian-iggy-streams) — Iggy transport backend
- [simian-reactor](#simian-reactor) — Tower middleware: `MakoReactor`, `MakoLayer`, `MakoBattery`
- [simian-event](#simian-event) — Event emitter/receiver system
- [simian-config](#simian-config) — Feature-gated configuration management
- [simian-tracing](#simian-tracing) — OpenTelemetry + Loki observability
- [simian-metrics](#simian-metrics) — Prometheus metrics exporter
- [simian-surreal-client](#simian-surreal-client) — SurrealDB client with Tower integration
- [simian-http-listener](#simian-http-listener) — Axum HTTP listener utilities
- [simian-llm](#simian-llm) — LLM client integration
- [Binaries](#binaries)
- [Tower Integration Deep Dive](#tower-integration-deep-dive)
- [Reactor Analysis & Cross-Crate Integration](#reactor-analysis--cross-crate-integration)
- [Build & Test](#build--test)

---

## Architecture Overview

```
┌──────────────────────────────────────────────────────────────────┐
│                        SimianAgent<T>                            │
│                      (simian-base-api)                           │
│  request() / inform() / reply() / listen()                       │
└──────────────┬───────────────┬───────────────┬───────────────────┘
               │               │               │
       ┌───────▼──────┐ ┌─────▼──────┐ ┌──────▼──────┐
       │ NatsTransport │ │IggyTransport│ │ <your impl> │
       │(nats-streams) │ │(iggy-streams)│ │             │
       └───────┬───────┘ └─────┬──────┘ └─────────────┘
               │               │
       ┌───────▼───────────────▼─────────────────────┐
       │        Transport trait (base-api)            │
       │  publish(AcpMessage) / subscribe(AgentId)    │
       └─────────────────────────────────────────────┘
```

The system separates **what agents say** (`AcpMessage`, `Performative`) from **how they say it** (`Transport` trait). On top of the transport layer, `simian-nats-streams` adds a secondary **Frame protocol** for structured RPC and a **Kong/Monkey** pattern for NATS-specific service hosting.

`simian-reactor` sits orthogonally as a **Tower middleware layer**, providing rate-limiting (`MakoBattery`), function registries (`ReactorCore`), and composable service wrapping (`MakoLayer`/`MakoReactor`).

---

## Crate Map

| Crate | Type | Purpose |
|---|---|---|
| `simian-base-api` | lib | Core ACP: `Transport`, `AcpMessage`, `SimianAgent` |
| `simian-nats-streams` | lib | NATS transport, Frame protocol, Kong/Monkey, Tower services |
| `simian-iggy-streams` | lib | Iggy transport backend |
| `simian-reactor` | lib | Tower middleware: `MakoReactor`, `MakoLayer`, `MakoBattery` |
| `simian-event` | lib+bin | Event emitter/receiver on top of transport |
| `simian-config` | lib | Feature-gated config (`autotask`, `jira`, `surreal`, `swimlane`) |
| `simian-tracing` | lib | OpenTelemetry, Loki, tracing integration |
| `simian-metrics` | lib | Prometheus metrics (HTTP listener + push gateway) |
| `simian-surreal-client` | lib | SurrealDB Tower service + live queries |
| `simian-http-listener` | lib | Axum-based HTTP server utilities |
| `simian-llm` | lib | LLM client protocol |
| `simian-bin-monkey` | bin | CLI tool to send Frame messages over NATS |
| `simian-bin-cfg-creator` | bin | Configuration file generator |

---

## simian-base-api

The foundational crate. Defines the protocol-level abstractions that all other crates build on.

### `Transport` trait

```rust
#[async_trait]
pub trait Transport: Send + Sync + Debug {
    async fn publish(&self, message: AcpMessage) -> Result<()>;
    async fn subscribe(&self, agent_id: &AgentId) -> Result<BoxStream<'static, AcpMessage>>;
    async fn is_connected(&self) -> bool;
}
```

Any messaging backend (NATS, Iggy, in-memory mock) implements this trait. The trait is `Send + Sync + Debug` so it can be shared across tasks and inspected at runtime.

### `AcpMessage`

```rust
pub struct AcpMessage {
    pub message_id: Uuid,
    pub source: AgentId,
    pub target: Option<AgentId>,  // None = broadcast
    pub performative: Performative,
    pub subject: String,
    pub conversation_id: Uuid,
    pub payload: Bytes,
    pub timestamp: i64,
}
```

Performatives: `Request`, `Inform`, `Query`, `Agree`, `Refuse`, `Failure`, `Subscribe`, `Unsubscribe`.

### `SimianAgent<T: Transport>`

High-level wrapper hiding message construction:

```rust
let transport = NatsTransport::new("nats://localhost:4222", "agents").await?;
let agent = SimianAgent::new(AgentId::new("weather-bot"), transport);

// Send a request — returns conversation_id for correlation
let conv_id = agent.request(
    AgentId::new("time-service"),
    "get-time",
    Bytes::from(r#"{"tz":"America/New_York"}"#),
).await?;

// Broadcast (target = None)
agent.inform(None, "status", Bytes::from("online")).await?;

// Reply preserves conversation_id
agent.reply(&incoming_msg, Performative::Agree, Bytes::from("ack")).await?;

// Listen returns BoxStream<AcpMessage>
let mut stream = agent.listen().await?;
while let Some(msg) = stream.next().await {
    match msg.performative {
        Performative::Request => { /* handle */ },
        Performative::Inform => { /* handle */ },
        _ => {}
    }
}
```

---

## simian-nats-streams

NATS-specific transport plus a rich layer of Frame-based RPC tooling.

### NATS Transport

Implements `Transport` with subject patterns:
- Direct: `{base_subject}.{agent_id}.inbox`
- Broadcast: `{base_subject}.broadcast`

Merges both into a single `BoxStream` via `subscribe()`.

```rust
let transport = NatsTransport::new("nats://10.2.4.106:4222", "agents").await?;
```

### Frame Protocol

A secondary serialization layer (bincode) for structured NATS RPC, independent of `AcpMessage`:

```rust
pub enum Frame {
    Ping,
    Pong,
    Msg(String),
    Bytes(Box<[u8]>),
    SendBox(SendBox),
    Json(Value),
    Close,
    Exec(Proc),    // RPC: execute a named function with args
    Fin,
    Error(String),
}
```

`Proc` represents a remote procedure call:

```rust
pub struct Proc {
    pub cmd: String,
    pub args: Vec<String>,
}

// Create an exec frame
let frame = Frame::exec("get-time", vec!["America/New_York"]);
```

Frames implement `Encoder`/`Decoder` (bincode) and `From<Frame> for Bytes` / `From<Bytes> for Frame`.

### Kong / Monkey Pattern

**Kong** — a NATS subscriber that listens on a subject and dispatches incoming frames to a handler function or Tower service:

```rust
let kong = Kong::new("agents.time", "nats://localhost:4222").await?;

// Simple function handler
kong.service(|| async { Frame::pong() }).await?;

// Future-based handler (receives the incoming Frame)
kong.service_future(|frame: Frame| async move {
    Ok::<_, BoxError>(Frame::message("handled"))
}).await?;

// Tower service handler
kong.tower_service(my_tower_service).await?;
```

**Monkey** — a NATS publisher that sends frames and optionally awaits responses:

```rust
let monkey = Monkey::new("agents.time", "nats://localhost:4222").await;

// Fire-and-forget
monkey.publish(Frame::ping()).await?;

// Request-reply
let response: async_nats::Message = monkey.msg(Frame::ping()).await?;

// Request-reply with timeout
let response = monkey.msg_timeout(
    Frame::exec("get-time", vec!["UTC"]),
    Some(Duration::from_secs(30)),
).await?;
```

**KingKong** — orchestrator that manages multiple Kongs under a single NATS subject prefix, with health checking and cancellation:

```rust
let mut kk = KingKong::new("services", "nats://localhost:4222", "0.0.0.0:8080").await;

// Register simple handler on `services.health`
kk.new_kong("health", || async { Frame::pong() }).await?;

// Register Tower service on `services.time`
kk.new_tower_kong("time", my_time_service).await?;

// Health check all registered kongs (sends Ping, expects Pong)
assert!(kk.health().await);

// Block until cancellation or health failure
kk.wait().await?;
```

#### KingKong as `tower::Service<Frame>` (NEW)

`KingKong` now implements `Service<Frame>`, enabling it to be composed into Tower service stacks. When called, it uses its internal `Monkey` to publish the frame and decode the response:

```rust
impl Service<Frame> for KingKong {
    type Response = Frame;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<Frame, BoxError>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Frame) -> Self::Future {
        let monkey = self.monkey.clone();
        Box::pin(async move {
            let resp = monkey.msg(req).await.map_err(|e| anyhow!(e))?;
            let frame = Frame::decode(&resp.payload).map_err(|e| anyhow!(e))?;
            Ok(frame)
        })
    }
}
```

This means a `KingKong` can now be wrapped with Tower layers:

```rust
use tower::ServiceBuilder;

let kk = KingKong::new("services", NATS_ADDR, "0.0.0.0:8080").await;

let svc = ServiceBuilder::new()
    .buffer(10)
    .concurrency_limit(5)
    .timeout(Duration::from_secs(30))
    .service(kk);

// Use as any other Tower service
let response: Frame = svc.ready().await?.call(Frame::ping()).await?;
```

### Tower Services in simian-nats-streams

**`NatsSend<S, Request>`** — A `Service` that forwards a request to NATS (via Monkey), then passes the response to an inner service:

```rust
let inner = MyProcessingService;
let mut svc = NatsSend::new("agents.time", "nats://localhost:4222", inner);
let result: Bytes = svc.ready().await?.call(my_request).await?;
```

**`NatsLayer<S, Request>`** — Tower `Layer` that wraps any service with `NatsSend`:

```rust
let svc = ServiceBuilder::new()
    .layer(NatsLayer::new("agents.time", "nats://localhost:4222"))
    .service(my_inner_service);
```

---

## simian-iggy-streams

Alternative `Transport` implementation using the Iggy streaming platform. Follows the same trait contract as `NatsTransport`.

---

## simian-reactor

Tower-based middleware for composable request/response processing. This crate provides three main components.

### `MakoBattery` — Semaphore-based Rate Limiter

A token-bucket rate limiter using `tokio::sync::Semaphore`. Permits are consumed on `acquire()` and replenished on a timer:

```rust
use simian_reactor::MakoBattery;
use tokio::time::Duration;

// 5 permits, replenished 1 per 200ms
let battery = MakoBattery::new(Duration::from_millis(200), 5);

// Blocks if no permits available
battery.acquire().await;
do_work().await;
```

Internally:
- The semaphore starts at `capacity` permits.
- A background `tokio::spawn` task ticks at the specified `Duration`. Each tick adds 1 permit if below capacity.
- `acquire()` calls `sem.acquire().await` and then `permit.forget()` so the permit is consumed (not auto-released on drop).
- Dropping the `MakoBattery` aborts the background replenishment task.

### `ReactorCore` — Named Function Registry

Stores `fn(Vec<String>) -> F` handlers keyed by name. Used by `MakoReactor` to dispatch `Proc` (remote procedure calls from the Frame protocol):

```rust
// Internally:
core.register_function("get-time", |args: Vec<String>| async move {
    let tz = &args[0];
    let resp = reqwest::get(format!("http://worldtimeapi.org/api/timezone/{}", tz)).await?;
    Ok(Frame::message(resp.text().await?.as_str()))
})?;

// Dispatch
let frame = core.call_registered_function(Proc::new("get-time", vec!["UTC"])).await?;
```

### `MakoReactor<S, F, R, E>` — Service Wrapper with Registry

Wraps an inner `tower::Service` and a `ReactorCore`. Can route `Frame::Exec(proc)` to registered functions or delegate to the inner service:

```rust
pub struct MakoReactor<S, F, R, E> {
    pub core: Arc<Mutex<ReactorCore<F, R, E>>>,
    pub service: S,
}
```

Implements `Service<T>` where `T: Into<Bytes>` — decodes the incoming bytes into a `Frame`, and if it's `Frame::Exec(proc)`, routes to the registered function. Otherwise delegates to the inner service.

### `MakoLayer` — Tower Layer producing `MakoReactor`

Standard Tower `Layer` implementation that wraps any service with a `MakoReactor`:

```rust
use simian_reactor::MakoLayer;
use tower::ServiceBuilder;

let svc = ServiceBuilder::new()
    .buffer(10)
    .concurrency_limit(5)
    .timeout(Duration::from_secs(1))
    .rate_limit(1, Duration::from_secs(1))
    .layer(MakoLayer::<Frame>::new(120, 120))
    .service(FrameHandler);
```

### `FrameFuture<F, R, E>` — Pin-projected Future wrapper

Wraps any `Future<Output = Result<R, E>>` with `pin_project_lite` for ergonomic use as a `Service::Future`:

```rust
impl Service<Frame> for FrameHandler {
    type Response = Frame;
    type Future = FrameFuture<Pin<Box<dyn Future<Output = Result<Frame, BoxError>> + Send + Sync>>, Frame, BoxError>;
    type Error = BoxError;

    fn call(&mut self, req: Frame) -> Self::Future {
        FrameFuture::new(Box::pin(async { Ok(handle(req).await?) }))
    }
}
```

### Full Example: Reactor + KingKong

From [simian-reactor/examples/mako.rs](simian-reactor/examples/mako.rs):

```rust
use simian_nats_streams::{Frame, KingKong, Monkey};
use simian_reactor::FrameFuture;
use tower::{BoxError, Service, ServiceExt};

async fn call_time(args: Vec<String>) -> Result<Frame, BoxError> {
    let endpoint = &args[0];
    let resp = reqwest::get(format!("http://worldtimeapi.org/api/timezone/{}", endpoint)).await?;
    let val: serde_json::Value = resp.json().await?;
    Ok(Frame::message(serde_json::to_string(&val)?.as_str()))
}

async fn frame_handler(frame: Frame) -> Result<Frame, BoxError> {
    match frame {
        Frame::Ping => Ok(Frame::pong()),
        Frame::Exec(proc) => call_time(proc.args).await,
        _ => unimplemented!(),
    }
}

#[derive(Debug)]
struct FrameHandler;

impl Service<Frame> for FrameHandler {
    type Response = Frame;
    type Future = FrameFuture<
        Pin<Box<dyn Future<Output = Result<Frame, BoxError>> + Send + Sync>>,
        Frame, BoxError,
    >;
    type Error = BoxError;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), BoxError>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Frame) -> Self::Future {
        FrameFuture::new(Box::pin(async { Ok(frame_handler(req).await?) }))
    }
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    // Compose with Tower middleware
    let srv = tower::ServiceBuilder::new()
        .buffer(10)
        .concurrency_limit(5)
        .timeout(tokio::time::Duration::from_secs(1))
        .rate_limit(1, tokio::time::Duration::from_secs(1))
        .service(FrameHandler);

    let reactor_service = Arc::new(Mutex::new(srv));

    // Host via KingKong
    let mut kkong = KingKong::new("time", "nats://10.2.4.106:4222", "0.0.0.0:8080").await;
    kkong.new_tower_kong("time", reactor_service.clone()).await?;

    // Call via Monkey
    let monkey = Monkey::new("time.time", "nats://10.2.4.106:4222").await;
    let resp = monkey.msg(Frame::exec("time", vec!["America/New_York"])).await?;
    println!("Response: {:?}", resp);

    Ok(())
}
```

---

## Reactor Analysis & Cross-Crate Integration

### Overlap Assessment

| Capability | simian-reactor | simian-nats-streams | Overlap? |
|---|---|---|---|
| Tower `Service` impl | `MakoReactor` (generic) | `NatsSend`, `KingKong` (NATS-specific) | Complementary |
| Tower `Layer` | `MakoLayer` | `NatsLayer` | Complementary |
| Rate limiting | `MakoBattery` (semaphore) | `tower::rate_limit` via ServiceBuilder | Partial — Battery is lower-level |
| Function registry | `ReactorCore` (name→fn map) | Kong dispatches to fn pointers | Overlapping concept |
| Frame dispatch | `mako_service.rs` decodes Frame→Exec→registry | `tower_service.rs` decodes Frame→Service::call | Overlapping |

### Key Findings

1. **`MakoBattery` is unique and reusable.** It implements time-windowed token-bucket rate limiting at the semaphore level, which is more granular than Tower's built-in `rate_limit`. It could be used inside `SimianAgent` to throttle outbound messages, or anywhere a custom backpressure mechanism is needed.

2. **`ReactorCore` overlaps with Kong's dispatch pattern.** Both maintain a mapping from names to handler functions. Kong does it at the NATS subscription level; ReactorCore does it inside a Tower service. Consider unifying: Kong could delegate to a ReactorCore internally instead of maintaining its own function dispatch.

3. **`MakoReactor`'s `Service` impl for Frame is tightly coupled to `simian-nats-streams`.** The `mako_service.rs` directly imports `Frame` and `Decoder` from `simian-nats-streams`. This coupling means the reactor can't be used with `AcpMessage` or other frame types without refactoring.

4. **KingKong now implements `Service<Frame>`.** This new implementation allows KingKong to participate in Tower service composition, closing the gap between the reactor's generic middleware approach and KingKong's NATS-specific orchestration.

### Integration Roadmap

```
Phase 1 (DONE):   KingKong implements Service<Frame>
Phase 2 (PLANNED): Extract MakoBattery into SimianAgent (feature-gated)
Phase 3 (PLANNED): Unify ReactorCore + Kong dispatch into shared registry
Phase 4 (PLANNED): Make Frame protocol transport-agnostic (move to base-api)
```

---

## simian-event

Event emitter/receiver system built on the transport layer:
- `Emitter` wraps a transport to publish typed events
- `SimianEvent` / `SimianEventType` provide structured event abstractions

---

## simian-config

Feature-gated configuration using the `config` crate. Each integration has its own feature flag:

```toml
[features]
dev = []
surreal = []
autotask = []
jira = []
swimlane = []
```

Configuration structs use `serde` and are loaded with environment variable prefixes:

```rust
// AT_URL, AT_INTEGRATION_CODE, AT_USERNAME, AT_PASSWORD
let cfg: AutotaskCfg = autotask_config()?;
```

---

## simian-tracing

Observability stack integrating:
- `tracing` + `tracing-opentelemetry` for structured logging and distributed tracing
- Loki push for log aggregation
- Sentry layer for error reporting
- Batch configuration for tuning export intervals

---

## simian-metrics

Prometheus metrics exporter with:
- HTTP listener endpoint (`/metrics`)
- Push gateway support for environments without inbound access

---

## simian-surreal-client

SurrealDB client with:
- `Storable` trait for typed record operations
- Tower `Service` integration for request/response patterns
- Live query support via `live_select`
- Connection credential management

---

## simian-http-listener

Axum-based HTTP server utilities:
- `Server` struct with bind address management
- Graceful shutdown support
- Timer utilities

---

## simian-llm

LLM client library:
- Protocol definitions for LLM interaction
- LM Studio backend support
- NATS integration tests for distributed LLM calling

---

## Binaries

### simian-bin-monkey

CLI tool for sending Frame-encoded messages to NATS subjects. Useful for testing Kong services:

```bash
cargo run -p simian-bin-monkey -- --subject agents.time --nats nats://localhost:4222
```

### simian-bin-cfg-creator

Generates configuration files for the simian ecosystem.

---

## Build & Test

```bash
# Build entire workspace
cargo build --workspace

# Build specific package
cargo build -p simian-reactor

# Run all tests
cargo test --workspace --verbose

# Run tests for a specific crate
cargo test -p simian-nats-streams

# Run a specific test
cargo test -p simian-reactor test_battery_capacity

# Format
cargo fmt

# Check without building
cargo check --workspace

# Clippy
cargo clippy --workspace
```

### Workspace Dependencies

All shared dependencies are centralized in the root `Cargo.toml` under `[workspace.dependencies]`. Individual crates reference them with `workspace = true`:

```toml
# Root Cargo.toml
[workspace.dependencies]
tower = { version = "0.5.1", features = ["buffer"] }
tokio = { version = "1.37.0", features = ["rt-multi-thread", "macros", "net", "signal"] }

# Crate Cargo.toml
[dependencies]
tower = { workspace = true }
tokio = { workspace = true }
```