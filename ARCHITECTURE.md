# simian-bastion Architecture

This document provides a deep dive into the design, structure, and implementation of the simian-bastion agent framework. It explains **why** the system is organized as it is, how different layers interact, and how to extend the framework.

---

## Table of Contents

- [Core Design Principles](#core-design-principles)
- [Layer Architecture](#layer-architecture)
- [Agent Communication Protocol (ACP)](#agent-communication-protocol-acp)
- [Frame Protocol](#frame-protocol)
- [Transport Implementations](#transport-implementations)
- [Kong/Monkey Pattern](#kongmonkey-pattern)
- [Tower Middleware Integration](#tower-middleware-integration)
- [Observability & Configuration](#observability--configuration)
- [Crate Dependency Graph](#crate-dependency-graph)
- [Message Flow Examples](#message-flow-examples)
- [Extension Guide](#extension-guide)
- [Design Decisions & Rationale](#design-decisions--rationale)

---

## Core Design Principles

The simian-bastion framework is built on four foundational principles:

### 1. **Separation of Concerns**
- **What agents say** (ACP protocol) is separate from **how they say it** (Transport trait)
- **Message semantics** (Performatives: Request, Inform, Query) are independent of **serialization format**
- **Business logic** is decoupled from **network topology**

### 2. **Pluggable Everything**
- Any messaging backend can implement `Transport` (NATS, Iggy, in-memory, etc.)
- Any middleware can wrap services via Tower's `Layer` pattern
- Any observability system can plug into the tracing/metrics hooks

### 3. **Composability via Tower**
- Services are composable functions: `Fn(Request) -> Future<Response>`
- Middleware adds cross-cutting concerns (rate limiting, logging, retries)
- Layers stack cleanly without touching business logic

### 4. **Zero-Copy Where Possible**
- `Bytes` type used throughout for payload handling
- `rkyv` serialization for Frame protocol (zero-copy deserialization)
- Streaming via `BoxStream` to avoid buffering entire message sets

---

## Layer Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         APPLICATION LAYER                               │
│  Your agents, services, business logic built with SimianAgent<T>        │
└────────────────────────┬────────────────────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────────────────────┐
│                      AGENT ABSTRACTION LAYER                            │
│              SimianAgent<T: Transport>  (simian-base-api)               │
│                                                                          │
│  Methods: request(), inform(), reply(), listen()                        │
│  Manages: Message IDs, conversation tracking, performatives             │
└─────────────┬──────────────────────────────────────────────────────────┘
              │
┌─────────────▼──────────────────────────────────────────────────────────┐
│                    PROTOCOL LAYER (ACP)                                 │
│                  AcpMessage + Performative                              │
│                                                                          │
│  struct AcpMessage {                                                    │
│    message_id: Uuid,                                                    │
│    source: AgentId,                                                     │
│    target: Option<AgentId>,  // None = broadcast                       │
│    performative: Performative,                                          │
│    subject: String,                                                     │
│    conversation_id: Uuid,                                               │
│    payload: Bytes,                                                      │
│    timestamp: i64,                                                      │
│  }                                                                      │
└──────────────┬─────────────────────────────────────────────────────────┘
               │
┌──────────────▼─────────────────────────────────────────────────────────┐
│                    TRANSPORT ABSTRACTION LAYER                          │
│              Transport trait (simian-base-api)                          │
│                                                                          │
│  trait Transport: Send + Sync + Debug {                                │
│    async fn publish(&self, AcpMessage) -> Result<()>;                  │
│    async fn subscribe(&self, AgentId) -> BoxStream<AcpMessage>;        │
│    async fn is_connected(&self) -> bool;                               │
│  }                                                                      │
└──┬───────────────────────┬───────────────────────┬──────────────────────┘
   │                       │                       │
┌──▼──────────────┐  ┌────▼──────────────┐  ┌────▼──────────────┐
│ NatsTransport   │  │ IggyTransport     │  │ <Your Transport>  │
│(nats-streams)   │  │(iggy-streams)     │  │                   │
└─────────────────┘  └───────────────────┘  └───────────────────┘
         │                     │                      │
┌────────▼─────────────────────▼──────────────────────▼──────────────────┐
│                    NETWORK/INFRASTRUCTURE LAYER                         │
│              NATS / Iggy / Kafka / Redis / In-Memory                    │
└─────────────────────────────────────────────────────────────────────────┘


┌─────────────────────────────────────────────────────────────────────────┐
│                      ORTHOGONAL CONCERNS                                │
│  (These layers work alongside the main stack)                           │
│                                                                          │
│  ┌───────────────────────┐  ┌─────────────────────────────────┐        │
│  │  Frame Protocol       │  │  Tower Middleware               │        │
│  │  (simian-reactor)     │  │  (simian-reactor)               │        │
│  │                       │  │                                 │        │
│  │  - rkyv serialization │  │  - MakoBattery (rate limiting)  │        │
│  │  - Exec/Proc RPC      │  │  - MakoLayer (wrapping)         │        │
│  │  - Kong/Monkey pattern│  │  - ReactorCore (registry)       │        │
│  └───────────────────────┘  └─────────────────────────────────┘        │
│                                                                          │
│  ┌───────────────────────┐  ┌─────────────────────────────────┐        │
│  │  Observability        │  │  Configuration                  │        │
│  │  (simian-tracing)     │  │  (simian-config)                │        │
│  │  (simian-metrics)     │  │                                 │        │
│  │                       │  │  - Feature-gated configs        │        │
│  │  - OpenTelemetry      │  │  - Environment variable loading │        │
│  │  - Loki logging       │  │  - cfg-gen binary               │        │
│  │  - Prometheus metrics │  │                                 │        │
│  └───────────────────────┘  └─────────────────────────────────┘        │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Agent Communication Protocol (ACP)

ACP is the **semantic layer** for agent interaction. It defines **what** agents are trying to communicate, independent of **how** the bytes are transmitted.

### Performatives

Inspired by FIPA-ACL, performatives express the **intent** of a message:

| Performative | Meaning | Example Use Case |
|--------------|---------|------------------|
| `Request` | Asking another agent to perform an action | "Execute this task for me" |
| `Inform` | Broadcasting information without expecting action | "I am now online", "Temperature is 72°F" |
| `Query` | Requesting information | "What is your current status?" |
| `Agree` | Agreeing to perform a requested action | "I will start that task" |
| `Refuse` | Declining a request | "I cannot fulfill that request" |
| `Failure` | Reporting that an agreed action failed | "Task execution failed with error X" |
| `Subscribe` | Requesting to receive future messages on a topic | "Notify me of all temperature changes" |
| `Unsubscribe` | Stopping a subscription | "Stop temperature notifications" |

### Message Structure

```rust
pub struct AcpMessage {
    pub message_id: Uuid,           // Unique ID for this message
    pub source: AgentId,            // Who sent it
    pub target: Option<AgentId>,    // Who should receive it (None = broadcast)
    pub performative: Performative, // Intent of the message
    pub subject: String,            // Topic/subject line
    pub conversation_id: Uuid,      // Thread/conversation grouping
    pub payload: Bytes,             // Actual data (format is application-specific)
    pub timestamp: i64,             // Unix timestamp
}
```

### Conversation Tracking

Each `AcpMessage` has a `conversation_id` that groups related messages:

1. **Agent A** sends a `Request` with a new `conversation_id`
2. **Agent B** replies with `Agree` using the **same** `conversation_id`
3. Later, **Agent B** sends `Inform` with results, preserving `conversation_id`
4. **Agent A** can correlate all messages in this request/response flow

### Broadcasting

Setting `target: None` broadcasts the message to all subscribed agents. Transport implementations merge:
- Direct messages: `{base_subject}.{agent_id}.inbox`
- Broadcast messages: `{base_subject}.broadcast`

---

## Frame Protocol

The **Frame protocol** is a **secondary serialization layer** built on top of transports, specifically designed for **structured RPC** and **typed message passing** in NATS-based systems.

### Why Separate Frame from ACP?

| Concern | ACP | Frame Protocol |
|---------|-----|----------------|
| **Purpose** | High-level agent semantics | Low-level RPC mechanism |
| **Scope** | Transport-agnostic | NATS-optimized |
| **Serialization** | JSON (via serde) | rkyv (zero-copy binary) |
| **Use Case** | Agent-to-agent conversations | Service invocation, health checks |
| **Data Model** | `Performative` + `payload: Bytes` | Typed enum with `Exec(Proc)` |

**Design Rationale:**
- ACP is for **agents** talking to each other across any transport
- Frame is for **services** doing RPC over NATS (Ping/Pong, Exec, SendBox)
- Separating them keeps ACP simple and Frame optimized for performance

### Frame Variants

```rust
pub enum Frame {
    Ping,                    // Health check request
    Pong,                    // Health check response
    Msg(String),             // Simple text message
    Bytes(Box<[u8]>),        // Binary data
    Json(JsonValue),         // JSON payload (wrapped for rkyv)
    SendBox(SendBox),        // Addressed message with envelope
    Exec(Proc),              // Remote procedure call
    Fin,                     // Signal completion
    Close,                   // Close connection/stream
    Error(String),           // Error message
}
```

### Proc: Remote Procedure Calls

```rust
pub struct Proc {
    pub cmd: String,         // Function name to invoke
    pub args: Vec<String>,   // Arguments to pass
}

// Example: Call a time service
let frame = Frame::exec("get-time", vec!["America/New_York"]);
```

The `Exec` variant enables **function dispatch** over NATS without defining request/response types upfront.

### Serialization: rkyv

Frame uses `rkyv` (archive) for serialization:
- **Zero-copy deserialization**: No allocation for simple reads
- **Validation**: Built-in bounds checking
- **Performance**: Faster than bincode/serde for read-heavy workloads

```rust
impl Frame {
    pub fn encode(&self) -> Result<Vec<u8>, BoxError> {
        // rkyv serialization
    }
    
    pub fn decode(bytes: &[u8]) -> Result<Frame, BoxError> {
        // rkyv deserialization
    }
}
```

---

## Transport Implementations

### NatsTransport

Implements `Transport` using the NATS messaging system.

**Subject Pattern:**
```
{base_subject}.{agent_id}.inbox      → Direct messages to agent
{base_subject}.broadcast             → Broadcast to all agents
```

**Implementation Highlights:**
```rust
pub struct NatsTransport {
    client: async_nats::Client,
    base_subject: String,
}

impl Transport for NatsTransport {
    async fn publish(&self, message: AcpMessage) -> Result<()> {
        let subject = if let Some(target) = &message.target {
            format!("{}.{}.inbox", self.base_subject, target.0)
        } else {
            format!("{}.broadcast", self.base_subject)
        };
        
        let payload = serde_json::to_vec(&message)?;
        self.client.publish(subject, payload.into()).await?;
        Ok(())
    }
    
    async fn subscribe(&self, agent_id: &AgentId) -> Result<BoxStream<'static, AcpMessage>> {
        let inbox_sub = self.client.subscribe(
            format!("{}.{}.inbox", self.base_subject, agent_id.0)
        ).await?;
        
        let broadcast_sub = self.client.subscribe(
            format!("{}.broadcast", self.base_subject)
        ).await?;
        
        // Merge both streams into one
        let merged = futures::stream::select(inbox_sub, broadcast_sub)
            .filter_map(|msg| async move {
                serde_json::from_slice::<AcpMessage>(&msg.payload).ok()
            });
        
        Ok(merged.boxed())
    }
}
```

### IggyTransport

Alternative transport using the Iggy streaming platform. Follows the same `Transport` contract but uses Iggy's stream/partition model instead of NATS subjects.

**Why Multiple Transports?**
- **Flexibility**: Different deployment environments have different infrastructure
- **Cost**: NATS might be overkill for local dev; in-memory works fine
- **Features**: Iggy provides message persistence; NATS provides clustering
- **Testing**: Mock transport for unit tests

---

## Kong/Monkey Pattern

The Kong/Monkey pattern is a **NATS-specific RPC framework** built on the Frame protocol. It separates **service hosting** (Kong) from **service invocation** (Monkey).

### Monkey (Client)

A **Monkey** publishes Frame messages to a NATS subject and optionally waits for responses.

```rust
pub struct Monkey {
    client: async_nats::Client,
    subject: String,
}

impl Monkey {
    // Fire-and-forget
    pub async fn publish(&self, frame: Frame) -> Result<()> {
        let bytes = frame.encode()?;
        self.client.publish(&self.subject, bytes.into()).await?;
        Ok(())
    }
    
    // Request-reply
    pub async fn msg(&self, frame: Frame) -> Result<async_nats::Message> {
        let bytes = frame.encode()?;
        let response = self.client.request(&self.subject, bytes.into()).await?;
        Ok(response)
    }
    
    // Request-reply with timeout
    pub async fn msg_timeout(
        &self,
        frame: Frame,
        timeout: Option<Duration>,
    ) -> Result<async_nats::Message> {
        let bytes = frame.encode()?;
        let timeout = timeout.unwrap_or(Duration::from_secs(30));
        
        let response = tokio::time::timeout(
            timeout,
            self.client.request(&self.subject, bytes.into())
        ).await??;
        
        Ok(response)
    }
}
```

### Kong (Server)

A **Kong** listens on a NATS subject and dispatches incoming Frame messages to a handler.

```rust
pub struct Kong {
    subject: String,
    subscriber: async_nats::Subscriber,
}

impl Kong {
    // Simple function handler
    pub async fn service<F, Fut>(&self, handler: F) -> Result<()>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Frame>,
    {
        while let Some(msg) = self.subscriber.next().await {
            let response_frame = handler().await;
            let response_bytes = response_frame.encode()?;
            
            if let Some(reply_to) = msg.reply {
                self.subscriber.publish(reply_to, response_bytes.into()).await?;
            }
        }
        Ok(())
    }
    
    // Future-based handler (receives Frame)
    pub async fn service_future<F, Fut>(&self, handler: F) -> Result<()>
    where
        F: Fn(Frame) -> Fut,
        Fut: Future<Output = Result<Frame, BoxError>>,
    {
        while let Some(msg) = self.subscriber.next().await {
            let frame = Frame::decode(&msg.payload)?;
            let response_frame = handler(frame).await?;
            let response_bytes = response_frame.encode()?;
            
            if let Some(reply_to) = msg.reply {
                self.subscriber.publish(reply_to, response_bytes.into()).await?;
            }
        }
        Ok(())
    }
    
    // Tower service handler
    pub async fn tower_service<S>(&self, service: Arc<Mutex<S>>) -> Result<()>
    where
        S: Service<Frame, Response = Frame, Error = BoxError>,
    {
        while let Some(msg) = self.subscriber.next().await {
            let frame = Frame::decode(&msg.payload)?;
            
            let mut svc = service.lock().await;
            let response_frame = svc.call(frame).await?;
            let response_bytes = response_frame.encode()?;
            
            if let Some(reply_to) = msg.reply {
                self.subscriber.publish(reply_to, response_bytes.into()).await?;
            }
        }
        Ok(())
    }
}
```

### KingKong (Orchestrator)

**KingKong** manages multiple Kong services under a unified subject prefix.

```rust
pub struct KingKong {
    pub name: String,
    pub subject: String,              // Base subject prefix
    nats_addr: String,
    addr_table: HashMap<String, String>,  // service_name -> full_subject
    abort_handles: Vec<AbortHandle>,
    listeners: JoinSet<Result<JoinHandleResult, BoxError>>,
    cancel_token: CancellationToken,
    http_listener: Server,            // Health check HTTP endpoint
    monkey: Monkey,                   // Internal client for health checks
}

impl KingKong {
    pub async fn new(subject: &str, nats_addr: &str, health_bind: &str) -> Self {
        // Creates a KingKong with auto-generated petname
        // Registers health check Kong at {subject}-health
    }
    
    // Register a simple function handler
    pub async fn new_kong<F, Fut>(
        &mut self,
        service_name: &str,
        handler: F,
    ) -> Result<()>
    where
        F: Fn() -> Fut + Send + 'static,
        Fut: Future<Output = Frame> + Send + 'static,
    {
        let full_subject = format!("{}.{}", self.subject, service_name);
        let kong = Kong::new(&full_subject, &self.nats_addr).await?;
        
        // Spawn Kong listener in background
        let handle = tokio::spawn(async move {
            kong.service(handler).await
        });
        
        self.addr_table.insert(service_name.to_string(), full_subject);
        self.abort_handles.push(handle.abort_handle());
        Ok(())
    }
    
    // Register a Tower service handler
    pub async fn new_tower_kong<S>(
        &mut self,
        service_name: &str,
        service: Arc<Mutex<S>>,
    ) -> Result<()>
    where
        S: Service<Frame, Response = Frame, Error = BoxError> + Send + 'static,
    {
        let full_subject = format!("{}.{}", self.subject, service_name);
        let kong = Kong::new(&full_subject, &self.nats_addr).await?;
        
        let handle = tokio::spawn(async move {
            kong.tower_service(service).await
        });
        
        self.addr_table.insert(service_name.to_string(), full_subject);
        self.abort_handles.push(handle.abort_handle());
        Ok(())
    }
    
    // Health check all registered Kongs
    pub async fn health(&self) -> bool {
        for (subject, _name) in &self.addr_table {
            let mut monkey = self.monkey.clone();
            monkey.set_subject(subject)?;
            
            let response = monkey.msg_timeout(
                Frame::ping(),
                Some(Duration::from_secs(5))
            ).await;
            
            match response {
                Ok(msg) => {
                    let frame = Frame::decode(&msg.payload)?;
                    if frame != Frame::pong() {
                        return false;
                    }
                }
                Err(_) => return false,
            }
        }
        true
    }
    
    // Block until cancelled or health check fails
    pub async fn wait(&mut self) -> Result<()> {
        loop {
            tokio::select! {
                _ = self.cancel_token.cancelled() => {
                    return Ok(());
                }
                _ = tokio::time::sleep(Duration::from_secs(30)) => {
                    if !self.health().await {
                        return Err(anyhow::anyhow!("Health check failed"));
                    }
                }
            }
        }
    }
}
```

### KingKong as Tower Service (NEW)

**KingKong now implements `Service<Frame>`**, enabling it to participate in Tower service composition:

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
            let resp = monkey.msg(req).await?;
            let frame = Frame::decode(&resp.payload)?;
            Ok(frame)
        })
    }
}
```

**Usage with Tower Layers:**
```rust
use tower::ServiceBuilder;

let kk = KingKong::new("services", "nats://localhost:4222", "0.0.0.0:8080").await;

let svc = ServiceBuilder::new()
    .buffer(10)
    .concurrency_limit(5)
    .timeout(Duration::from_secs(30))
    .service(kk);

// Use as any Tower service
let response: Frame = svc.ready().await?.call(Frame::ping()).await?;
```

---

## Tower Middleware Integration

Tower is a **library for building composable network services**. It defines a core `Service` trait:

```rust
pub trait Service<Request> {
    type Response;
    type Error;
    type Future: Future<Output = Result<Self::Response, Self::Error>>;
    
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;
    fn call(&mut self, req: Request) -> Self::Future;
}
```

simian-bastion uses Tower to add **cross-cutting concerns** without modifying business logic.

### MakoBattery: Rate Limiting

A semaphore-based token-bucket rate limiter.

```rust
pub struct MakoBattery {
    sem: Arc<Semaphore>,
    jh: tokio::task::JoinHandle<()>,
}

impl MakoBattery {
    pub fn new(duration: Duration, capacity: usize) -> Self {
        let sem = Arc::new(Semaphore::new(capacity));
        
        // Background task replenishes permits
        let jh = tokio::spawn({
            let sem = sem.clone();
            let mut interval = tokio::time::interval(duration);
            interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
            
            async move {
                loop {
                    interval.tick().await;
                    if sem.available_permits() < capacity {
                        sem.add_permits(1);
                    }
                }
            }
        });
        
        Self { jh, sem }
    }
    
    pub async fn acquire(&self) {
        let permit = self.sem.acquire().await.unwrap();
        permit.forget();  // Consume the permit (don't auto-release)
    }
}
```

**Use Case:** Throttle outbound requests to respect API rate limits.

```rust
let battery = MakoBattery::new(Duration::from_millis(100), 10);

for _ in 0..100 {
    battery.acquire().await;  // Blocks if no permits available
    make_api_call().await;
}
```

### ReactorCore: Function Registry

Stores named handlers for RPC dispatch.

```rust
pub struct ReactorCore<F, R, E>
where
    F: Future<Output = Result<R, E>>,
    E: Into<BoxError>,
{
    name: String,
    request_limit: usize,
    request_limit_time: Duration,
    services: HashMap<String, fn(Vec<String>) -> F>,
}

impl ReactorCore {
    pub fn register_function(
        &mut self,
        service_name: &str,
        func: fn(Vec<String>) -> F,
    ) -> Result<()> {
        self.services.insert(service_name.to_string(), func);
        Ok(())
    }
    
    pub fn get_function(&self, service_name: &str) -> Result<fn(Vec<String>) -> F> {
        self.services.get(service_name).copied()
            .ok_or_else(|| anyhow!("No function named {}", service_name))
    }
    
    pub async fn call_registered_function(&self, proc: Proc) -> Result<Frame> {
        let func = self.get_function(&proc.cmd)?;
        let result = func(proc.args).await?;
        Ok(result)
    }
}
```

**Use Case:** Dynamically route `Frame::Exec(proc)` to registered handlers.

### MakoReactor: Service + Registry

Wraps an inner `Service` and a `ReactorCore` for routing.

```rust
pub struct MakoReactor<S, F, R, E> {
    pub core: Arc<Mutex<ReactorCore<F, R, E>>>,
    pub service: S,
}

impl<S, F, R, E> Service<T> for MakoReactor<S, F, R, E>
where
    S: Service<Frame, Response = Frame, Error = BoxError>,
    T: Into<Bytes>,
{
    type Response = Frame;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<Frame, BoxError>> + Send>>;
    
    fn call(&mut self, req: T) -> Self::Future {
        let bytes: Bytes = req.into();
        let frame = Frame::decode(&bytes)?;
        
        match frame {
            Frame::Exec(proc) => {
                // Route to registered function
                let core = self.core.clone();
                Box::pin(async move {
                    let core = core.lock().await;
                    core.call_registered_function(proc).await
                })
            }
            _ => {
                // Delegate to inner service
                Box::pin(self.service.call(frame))
            }
        }
    }
}
```

### MakoLayer: Layer Implementation

Standard Tower `Layer` that wraps any service with `MakoReactor`.

```rust
pub struct MakoLayer<F> {
    request_limit: usize,
    request_time: usize,
    _marker: PhantomData<F>,
}

impl<S, F, R, E> Layer<S> for MakoLayer<F>
where
    F: Future<Output = Result<R, E>>,
    E: Into<BoxError>,
{
    type Service = MakoReactor<S, F, R, E>;
    
    fn layer(&self, service: S) -> Self::Service {
        let core = ReactorCore::new(self.request_limit, self.request_time);
        MakoReactor {
            core: Arc::new(Mutex::new(core)),
            service,
        }
    }
}
```

### Full Tower Stack Example

```rust
use tower::ServiceBuilder;
use simian_reactor::MakoLayer;

let svc = ServiceBuilder::new()
    .buffer(10)                          // Buffer 10 requests
    .concurrency_limit(5)                // Max 5 concurrent requests
    .timeout(Duration::from_secs(30))    // 30s timeout per request
    .rate_limit(10, Duration::from_secs(1))  // 10 req/sec
    .layer(MakoLayer::new(120, 1))       // MakoReactor with registry
    .service(MyFrameHandler);

// All layers stack cleanly without modifying MyFrameHandler
```

---

## Observability & Configuration

### simian-tracing

Integrates OpenTelemetry, Loki, and Sentry for distributed tracing and logging.

**Features:**
- Structured logging with `tracing` crate
- Span correlation across service boundaries
- Loki push for centralized log aggregation
- Sentry error tracking with context

**Setup:**
```rust
use simian_tracing::init_tracing;

#[tokio::main]
async fn main() {
    init_tracing("my-service", "http://loki:3100").await;
    
    tracing::info!("Service starting");
    
    // Spans automatically propagate context
    my_function().await;
}

#[tracing::instrument]
async fn my_function() {
    tracing::debug!("Inside function");
}
```

### simian-metrics

Prometheus metrics exporter with HTTP listener and push gateway support.

**Features:**
- Counter, Gauge, Histogram metrics
- `/metrics` HTTP endpoint
- Push gateway for short-lived jobs

**Setup:**
```rust
use simian_metrics::{init_metrics, counter};

#[tokio::main]
async fn main() {
    init_metrics("0.0.0.0:9090").await;
    
    let request_counter = counter!("requests_total");
    request_counter.increment(1);
}
```

### simian-config

Feature-gated configuration management.

**Features:**
- Environment variable loading with prefixes
- Structured config types with `serde`
- Feature flags for optional integrations (SurrealDB, Jira, Autotask)

**Example (SurrealDB config):**
```rust
use simian_config::surreal_config;

// Loads from env vars: SURREAL_PATH, SURREAL_NS, SURREAL_DB, etc.
let cfg = surreal_config()?;

// Or use cfg-gen binary
// $ cargo run --bin cfg-gen -- surreal \
//     --path "ws://localhost:8000" \
//     --ns "production" \
//     --db "maindb" \
//     --user "admin" \
//     --pass "secret"
```

---

## Crate Dependency Graph

```
┌─────────────────────┐
│  simian-base-api    │  ← Foundation: Transport trait, AcpMessage, SimianAgent
└──────────┬──────────┘
           │
     ┌─────┴──────┬───────────────┬───────────────┐
     │            │               │               │
┌────▼────────┐  │          ┌────▼──────────┐   │
│simian-nats- │  │          │simian-iggy-   │   │
│  streams    │  │          │  streams      │   │
└─────────────┘  │          └───────────────┘   │
                 │                               │
           ┌─────▼──────────┐            ┌──────▼──────────┐
           │ simian-reactor │            │ simian-event    │
           │                │            └─────────────────┘
           │ - Frame protocol│
           │ - MakoBattery   │
           │ - MakoReactor   │
           │ - Tower layers  │
           └─────────────────┘
                 │
          ┌──────┴───────┬──────────────┐
          │              │              │
    ┌─────▼───────┐ ┌───▼──────────┐ ┌─▼────────────┐
    │simian-      │ │simian-       │ │simian-       │
    │ tracing     │ │ metrics      │ │ config       │
    └─────────────┘ └──────────────┘ └──────────────┘


┌──────────────────────────────────────────────────────────┐
│              Specialized Libraries                       │
├──────────────────────────────────────────────────────────┤
│ simian-surreal-client  │ SurrealDB Tower integration     │
│ simian-http-listener   │ Axum HTTP server utilities      │
│ simian-llm             │ LLM client (LMStudio, OpenAI)   │
└──────────────────────────────────────────────────────────┘


┌──────────────────────────────────────────────────────────┐
│                 Binaries                                 │
├──────────────────────────────────────────────────────────┤
│ simian-bin-monkey      │ CLI tool to send Frame messages │
│ simian-bin-cfg-creator │ Configuration generator         │
└──────────────────────────────────────────────────────────┘
```

**Dependency Principles:**
- `simian-base-api` has **zero dependencies** on other simian crates (pure foundation)
- Transport implementations depend only on `simian-base-api`
- Higher-level crates (reactor, event) can depend on transports
- Observability crates (tracing, metrics) have **no simian dependencies** (they're pure utilities)

---

## Message Flow Examples

### Example 1: Simple Request/Response

```
┌─────────────┐                                    ┌─────────────┐
│  Agent A    │                                    │  Agent B    │
│ (weather)   │                                    │ (location)  │
└──────┬──────┘                                    └──────┬──────┘
       │                                                  │
       │ 1. request("get-location", {user: "alice"})    │
       ├──────────────────────────────────────────────►  │
       │   AcpMessage {                                  │
       │     performative: Request,                      │
       │     conversation_id: conv_123,                  │
       │     target: Some("location"),                   │
       │     ...                                         │
       │   }                                             │
       │                                                  │
       │                2. reply(Agree)                  │
       │  ◄──────────────────────────────────────────────┤
       │   AcpMessage {                                  │
       │     performative: Agree,                        │
       │     conversation_id: conv_123,                  │
       │     ...                                         │
       │   }                                             │
       │                                                  │
       │             3. reply(Inform, result)            │
       │  ◄──────────────────────────────────────────────┤
       │   AcpMessage {                                  │
       │     performative: Inform,                       │
       │     conversation_id: conv_123,                  │
       │     payload: {"lat": 40.7, "lon": -74.0}       │
       │   }                                             │
       │                                                  │
```

**Under the hood (NATS transport):**
1. Agent A publishes to `agents.location.inbox`
2. Agent B subscribed to `agents.location.inbox`
3. Agent B replies to `agents.weather.inbox` (target = source of original message)

### Example 2: Broadcasting

```
┌────────────┐                                  ┌────────────┐
│  Agent A   │                                  │  Agent B   │
│ (monitor)  │                                  │ (logger)   │
└──────┬─────┘                                  └──────┬─────┘
       │                                               │
       │  inform(None, "alert", {severity: "high"})  │
       ├───────────────────────────────────────────► │
       │   Published to: agents.broadcast             │
       │                                               │
       │                                        ┌──────▼─────┐
       │                                        │  Agent C   │
       │                                        │ (dashboard)│
       │                                        └────────────┘
       │  All agents subscribed to broadcast receive message
```

### Example 3: Frame RPC via Kong/Monkey

```
┌──────────┐                                    ┌──────────────────┐
│  Client  │                                    │   KingKong       │
│ (Monkey) │                                    │ (time service)   │
└────┬─────┘                                    └────────┬─────────┘
     │                                                   │
     │  1. monkey.msg(Frame::exec("get-time", ["UTC"])) │
     ├───────────────────────────────────────────────►  │
     │   Published to: services.time                    │
     │                                                   │
     │                                      Kong receives Frame::Exec
     │                                      Looks up "get-time" handler
     │                                      Calls: get_time_fn(["UTC"])
     │                                                   │
     │            2. Frame::Json({time: "..."})         │
     │  ◄───────────────────────────────────────────────┤
     │   Reply via NATS request/reply                   │
     │                                                   │
```

**Key Difference:**
- **ACP** uses `Performative` and preserves `conversation_id` for correlation
- **Frame** uses `Exec(Proc)` for direct function dispatch

### Example 4: Tower Middleware Stack

```
Client Request
     │
     ▼
┌─────────────────────┐
│  Buffer Layer       │  ← Queues requests if service busy
└──────┬──────────────┘
       │
       ▼
┌─────────────────────┐
│  Rate Limit Layer   │  ← Enforces 10 req/sec
└──────┬──────────────┘
       │
       ▼
┌─────────────────────┐
│  Timeout Layer      │  ← Cancels after 30s
└──────┬──────────────┘
       │
       ▼
┌─────────────────────┐
│  MakoReactor        │  ← Routes Frame::Exec to registry
└──────┬──────────────┘
       │
       ├─ Frame::Exec? → ReactorCore.call_registered_function()
       │
       └─ Other frames → Inner Service
                              │
                              ▼
                        ┌─────────────┐
                        │  Business   │
                        │   Logic     │
                        └─────────────┘
```

---

## Extension Guide

### How to Add a New Transport

1. **Create a new crate** (e.g., `simian-kafka-streams`)

2. **Implement the `Transport` trait:**

```rust
use simian_base_api::{Transport, AcpMessage, AgentId};
use async_trait::async_trait;
use futures::stream::BoxStream;

#[derive(Clone, Debug)]
pub struct KafkaTransport {
    producer: rdkafka::producer::FutureProducer,
    consumer: rdkafka::consumer::StreamConsumer,
    topic_prefix: String,
}

#[async_trait]
impl Transport for KafkaTransport {
    async fn publish(&self, message: AcpMessage) -> anyhow::Result<()> {
        let topic = if let Some(target) = &message.target {
            format!("{}.{}", self.topic_prefix, target.0)
        } else {
            format!("{}.broadcast", self.topic_prefix)
        };
        
        let payload = serde_json::to_vec(&message)?;
        self.producer.send(
            FutureRecord::to(&topic).payload(&payload),
            Duration::from_secs(5)
        ).await?;
        
        Ok(())
    }
    
    async fn subscribe(&self, agent_id: &AgentId) -> anyhow::Result<BoxStream<'static, AcpMessage>> {
        // Subscribe to agent-specific and broadcast topics
        // Return merged stream
        todo!()
    }
    
    async fn is_connected(&self) -> bool {
        // Check Kafka cluster connectivity
        todo!()
    }
}
```

3. **Add tests:**

```rust
#[tokio::test]
async fn test_kafka_publish_subscribe() {
    let transport = KafkaTransport::new("localhost:9092", "agents").await.unwrap();
    let agent = SimianAgent::new(AgentId::new("test"), transport);
    
    // Test publish/subscribe
}
```

### How to Add a New Performative

If you need custom performatives beyond the FIPA set:

1. **Extend the enum** in `simian-base-api/src/transport.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Performative {
    // Existing...
    Request,
    Inform,
    Query,
    
    // New ones
    Propose,       // Propose a solution
    AcceptProposal, // Accept a proposal
    RejectProposal, // Reject a proposal
}
```

2. **Add convenience methods** to `SimianAgent`:

```rust
impl<T: Transport> SimianAgent<T> {
    pub async fn propose(&self, target: AgentId, subject: &str, payload: Bytes) -> Result<Uuid> {
        let conversation_id = Uuid::new_v4();
        let message = AcpMessage {
            message_id: Uuid::new_v4(),
            source: self.id.clone(),
            target: Some(target),
            performative: Performative::Propose,
            subject: subject.to_string(),
            conversation_id,
            payload,
            timestamp: Utc::now().timestamp(),
        };
        self.transport.publish(message).await?;
        Ok(conversation_id)
    }
}
```

### How to Add a New Frame Variant

1. **Extend the enum** in `simian-reactor/src/protocol/frame.rs`:

```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Archive, RkyvSerialize, RkyvDeserialize)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Frame {
    // Existing...
    Ping,
    Pong,
    Exec(Proc),
    
    // New variant
    Stream(StreamData),  // Streaming data chunk
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Archive, RkyvSerialize, RkyvDeserialize)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct StreamData {
    pub stream_id: String,
    pub chunk: Box<[u8]>,
    pub is_final: bool,
}
```

2. **Add constructor methods:**

```rust
impl Frame {
    pub fn stream(stream_id: &str, chunk: Box<[u8]>, is_final: bool) -> Frame {
        Frame::Stream(StreamData {
            stream_id: stream_id.to_string(),
            chunk,
            is_final,
        })
    }
}
```

3. **Handle in service logic:**

```rust
impl Service<Frame> for MyHandler {
    fn call(&mut self, req: Frame) -> Self::Future {
        Box::pin(async move {
            match req {
                Frame::Stream(data) => {
                    handle_stream_chunk(data).await?;
                    Ok(Frame::pong())
                }
                _ => Ok(Frame::Error("Unsupported".into()))
            }
        })
    }
}
```

### How to Add a New Tower Layer

1. **Define the layer struct:**

```rust
pub struct RetryLayer {
    max_retries: usize,
}

impl RetryLayer {
    pub fn new(max_retries: usize) -> Self {
        Self { max_retries }
    }
}
```

2. **Implement `Layer` trait:**

```rust
impl<S> Layer<S> for RetryLayer {
    type Service = RetryService<S>;
    
    fn layer(&self, inner: S) -> Self::Service {
        RetryService {
            inner,
            max_retries: self.max_retries,
        }
    }
}
```

3. **Implement `Service` on the wrapper:**

```rust
pub struct RetryService<S> {
    inner: S,
    max_retries: usize,
}

impl<S> Service<Frame> for RetryService<S>
where
    S: Service<Frame> + Clone,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;
    
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }
    
    fn call(&mut self, req: Frame) -> Self::Future {
        let mut inner = self.inner.clone();
        let max_retries = self.max_retries;
        
        Box::pin(async move {
            let mut attempts = 0;
            loop {
                match inner.ready().await?.call(req.clone()).await {
                    Ok(response) => return Ok(response),
                    Err(e) if attempts < max_retries => {
                        attempts += 1;
                        tokio::time::sleep(Duration::from_millis(100 * attempts)).await;
                    }
                    Err(e) => return Err(e),
                }
            }
        })
    }
}
```

4. **Use in a ServiceBuilder:**

```rust
let svc = ServiceBuilder::new()
    .layer(RetryLayer::new(3))
    .service(MyService);
```

---

## Design Decisions & Rationale

### Why Separate ACP from Frame Protocol?

**Decision:** Maintain two serialization layers (ACP with JSON, Frame with rkyv)

**Rationale:**
1. **Separation of concerns:** ACP is semantic (performatives, conversation tracking); Frame is mechanical (RPC, health checks)
2. **Performance:** Frame uses rkyv for zero-copy deserialization in hot paths (Kong services handling thousands of req/sec)
3. **Flexibility:** ACP is transport-agnostic (works over any backend); Frame is NATS-optimized
4. **Evolution:** Can add new Frame variants without breaking ACP contract

**Trade-off:** Slight complexity having two message types, but the benefits outweigh the cost.

---

### Why `Transport` Trait Instead of Direct NATS Coupling?

**Decision:** Define a `Transport` trait that any backend can implement

**Rationale:**
1. **Testability:** Can use mock transports in unit tests
2. **Flexibility:** Different environments use different infrastructure (NATS in prod, in-memory in dev)
3. **Migration path:** Can switch transports without rewriting agent logic
4. **Feature parity:** Iggy provides persistence, NATS provides clustering — agents don't need to know

**Trade-off:** Trait adds indirection, but it's minimal overhead compared to network I/O.

---

### Why Kong/Monkey Naming?

**Decision:** Use "Kong" for servers, "Monkey" for clients

**Rationale:**
1. **Memorable:** Easier to remember than "FrameServer" / "FrameClient"
2. **Analogy:** Kong (large, stationary) vs. Monkey (agile, mobile)
3. **KingKong:** Natural orchestrator name that manages multiple Kongs

**Trade-off:** Non-obvious to newcomers, but quickly intuitive with context.

---

### Why Tower Instead of Custom Middleware?

**Decision:** Use Tower's `Service` and `Layer` traits

**Rationale:**
1. **Ecosystem:** Tower is battle-tested (used by Tonic, Hyper, Axum)
2. **Composability:** Layers stack cleanly without boilerplate
3. **Reusability:** Can use off-the-shelf Tower middleware (buffer, timeout, rate_limit)
4. **Backpressure:** `poll_ready` provides natural flow control

**Trade-off:** Tower has a learning curve, but it's a one-time investment.

---

### Why `Bytes` Instead of `Vec<u8>`?

**Decision:** Use `bytes::Bytes` for payloads

**Rationale:**
1. **Zero-copy cloning:** `Bytes` is a reference-counted pointer — cloning is cheap
2. **Interop:** Hyper, Tonic, and most async I/O libraries use `Bytes`
3. **Streaming:** Can split/slice without allocation

**Trade-off:** Slightly less ergonomic than `Vec<u8>` for small payloads, but critical for performance at scale.

---

### Why Workspace-Level Dependencies?

**Decision:** Centralize dependency versions in root `Cargo.toml`

**Rationale:**
1. **Consistency:** All crates use the same version of `tokio`, `tower`, etc.
2. **Maintenance:** One place to update versions
3. **Compile time:** Cargo can better deduplicate and cache builds

**Example:**
```toml
# Root Cargo.toml
[workspace.dependencies]
tokio = { version = "1.37.0", features = ["full"] }

# Crate Cargo.toml
[dependencies]
tokio = { workspace = true }
```

---

### Why `rkyv` for Frame, not `bincode`?

**Decision:** Use `rkyv` for Frame serialization

**Rationale:**
1. **Zero-copy:** Can read fields directly from serialized buffer (no deserialization overhead)
2. **Validation:** Built-in bounds checking prevents buffer overruns
3. **Performance:** Benchmarks show 10-50x faster deserialization than bincode for read-heavy workloads

**Trade-off:** Slightly larger serialized size, and more complex type constraints (must derive `Archive`).

---

### Why `conversation_id` in AcpMessage?

**Decision:** Every message has a `conversation_id` UUID

**Rationale:**
1. **Correlation:** Link request → reply → inform across multiple messages
2. **Debugging:** Trace a conversation through logs and distributed tracing
3. **State machines:** Track conversation state (e.g., "waiting for Agree before proceeding")

**Trade-off:** 16 bytes overhead per message, but essential for multi-step protocols.

---

### Why `Option<AgentId>` for Target (Broadcasting)?

**Decision:** `target: None` means broadcast

**Rationale:**
1. **Simplicity:** No separate "broadcast" method — just `inform(None, ...)`
2. **Explicitness:** Clear intent (None = everyone)
3. **Type safety:** Can't accidentally broadcast by passing wrong agent ID

**Implementation:**
- NATS: Publish to `{base}.broadcast` subject
- Agents subscribe to both `{base}.{agent_id}.inbox` and `{base}.broadcast`

---

## Summary

simian-bastion is an **agent framework** built on:
1. **ACP** for high-level agent semantics (performatives, conversations)
2. **Transport trait** for pluggable messaging backends
3. **Frame protocol** for low-level RPC over NATS
4. **Kong/Monkey** for service hosting and invocation
5. **Tower middleware** for composable cross-cutting concerns

The system **separates what from how**, making it easy to:
- Swap transports (NATS → Iggy → Kafka)
- Add middleware (rate limiting, retries, circuit breakers)
- Extend protocols (new performatives, new frame variants)
- Observe behavior (tracing, metrics)

When building agents:
- Use **`SimianAgent<T>`** for high-level operations
- Use **ACP** for inter-agent conversations
- Use **Frame + Kong/Monkey** for NATS-based services
- Use **Tower layers** to add middleware

When extending the framework:
- Implement **`Transport`** for new backends
- Extend **`Performative`** for new message intents
- Extend **`Frame`** for new RPC patterns
- Implement **`Layer<S>`** for new middleware

---

**For more examples, see:**
- `simian-reactor/examples/` — Tower service patterns
- `simian-nats-streams/examples/` — Kong/Monkey RPC
- `simian-base-api/src/agent.rs` — SimianAgent tests
