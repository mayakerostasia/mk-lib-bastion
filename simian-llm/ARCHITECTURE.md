# simian-llm Architecture

This document provides a comprehensive overview of the `simian-llm` crate architecture, design decisions, and component interactions.

## Table of Contents

1. [High-Level Overview](#high-level-overview)
2. [Core Components](#core-components)
3. [Trait-Based Design](#trait-based-design)
4. [Message Flow](#message-flow)
5. [Integration Points](#integration-points)
6. [Concurrency Model](#concurrency-model)

## High-Level Overview

`simian-llm` provides a modular, trait-based abstraction layer for Large Language Models in the Bastion agent framework. It enables agents to:

- Execute LLM operations locally (completions, chat)
- Send LLM requests to other agents via ACP messaging
- Broadcast LLM requests to all agents
- Handle incoming LLM requests from other agents

### Design Philosophy

**Pluggability First**: Every major component is defined as a trait, allowing users to:
- Swap LLM backends (LMStudio → OpenAI → Ollama)
- Use different transport layers (NATS → Iggy → custom)
- Mock components for testing

**Protocol-First**: All agent-to-agent communication follows the Agent Communication Protocol (ACP) standard with well-defined subjects and performatives.

**Observability**: Full instrumentation via `tracing` (structured logs) and `metrics` (counters) for production visibility.

## Core Components

### 1. LlmClient Trait

**File:** `src/client.rs`

The fundamental trait defining LLM operations:

```rust
pub trait LlmClient: Send + Sync {
    async fn complete(&self, prompt: &str) -> Result<LlmResponse>;
    async fn chat(&self, messages: &[ChatMessage]) -> Result<LlmResponse>;
    async fn health_check(&self) -> Result<bool>;
    fn model_info(&self) -> &ModelInfo;
}
```

**Responsibility:** Encapsulate all direct LLM API interactions.

**Implementations:**
- `LmStudioClient` - OpenAI-compatible HTTP client (in `src/lmstudio/`)

**Design Rationale:**
- Async trait allows non-blocking I/O
- `health_check()` enables graceful degradation
- `model_info()` provides metadata without state mutation
- Send + Sync required for use in multi-threaded contexts

### 2. LlmAgent<T, L>

**File:** `src/agent.rs`

Combines a Transport and LlmClient to enable agent-to-agent communication:

```rust
pub struct LlmAgent<T: Transport, L: LlmClient> {
    agent_id: AgentId,
    transport: T,
    llm_client: Arc<L>,
}
```

**Methods:**
- `request_completion(target, prompt)` - Send reasoning request
- `request_chat(target, messages)` - Send chat request
- `broadcast_completion(prompt)` - Broadcast to all
- `handle_message(msg)` - Process incoming ACP messages

**Responsibility:** Orchestrate LLM operations across agent boundaries.

**Design Rationale:**
- Generic over Transport and LlmClient for maximum flexibility
- `Arc<L>` allows cheap cloning (LlmClient is expensive to clone due to HTTP connection pools)
- Automatic message routing based on ACP subjects
- Error replies automatically generated on LLM failure

### 3. Protocol Types

**File:** `src/protocol.rs`

Defines ACP message payloads for LLM operations:

```rust
pub struct LlmReasonRequest {
    pub prompt: String,
    pub context: Option<HashMap<String, String>>,
}

pub struct LlmReasonResponse {
    pub response: String,
    pub model: String,
    pub tokens_used: Option<TokenUsage>,
}

pub struct LlmChatRequest {
    pub messages: Vec<ChatMessage>,
    pub context: Option<HashMap<String, String>>,
}

pub struct LlmChatResponse {
    pub message: ChatMessage,
    pub model: String,
    pub tokens_used: Option<TokenUsage>,
}
```

**Well-Known Subjects:**
- `llm.reason.request` - Request for text completion
- `llm.reason.response` - Response with completion result
- `llm.chat.request` - Request for multi-turn chat
- `llm.chat.response` - Response with chat result

**Responsibility:** Define serializable message contracts.

### 4. Type System

**File:** `src/types.rs`

Core domain types:

```rust
pub struct LlmResponse {
    pub content: String,
    pub model: String,
    pub usage: Option<TokenUsage>,
}

pub struct TokenUsage {
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
}

pub enum ChatRole {
    System,
    User,
    Assistant,
}

pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

pub struct ModelInfo {
    pub name: String,
    pub provider: String,
    pub max_tokens: Option<u32>,
}
```

**Responsibility:** Provide consistent types across all LLM operations.

### 5. Error Handling

**File:** `src/error.rs`

Comprehensive error types using `thiserror`:

```rust
#[derive(Error, Debug)]
pub enum LlmError {
    #[error("Connection failed: {0}")]
    ConnectionError(String),
    
    #[error("LLM request failed: {0}")]
    RequestError(String),
    
    #[error("Invalid response: {0}")]
    ResponseError(String),
}
```

**Responsibility:** Enable typed error handling in user code.

## Trait-Based Design

The architecture revolves around two primary traits:

### Transport Trait (from simian-base-api)

```rust
pub trait Transport: Send + Sync + Debug {
    async fn publish(&self, message: AcpMessage) -> Result<()>;
    async fn subscribe(&self, agent_id: &AgentId) -> Result<BoxStream<'static, AcpMessage>>;
    async fn is_connected(&self) -> bool;
}
```

**Implementations in the ecosystem:**
- `NatsTransport` (simian-nats-streams) - NATS Jetstream
- `IggyTransport` (simian-iggy-streams) - Iggy streams
- Custom implementations for testing/private networks

### LlmClient Trait (defined in simian-llm)

```rust
pub trait LlmClient: Send + Sync {
    async fn complete(&self, prompt: &str) -> Result<LlmResponse>;
    async fn chat(&self, messages: &[ChatMessage]) -> Result<LlmResponse>;
    async fn health_check(&self) -> Result<bool>;
    fn model_info(&self) -> &ModelInfo;
}
```

**Current implementations:**
- `LmStudioClient` - OpenAI-compatible REST API

**Future implementations:**
- `OpenAiClient` - Official OpenAI API
- `OllamaClient` - Local inference
- `ClaudeClient` - Anthropic Claude
- `GeminiClient` - Google Gemini

## Message Flow

### Scenario 1: Completion Request

```
Agent-1 (Requester)          NATS/Transport          Agent-2 (Handler)
────────────────────         ──────────────          ────────────────

request_completion()
  │
  ├─ Create AcpMessage
  │  ├─ subject: "llm.reason.request"
  │  ├─ performative: Request
  │  ├─ payload: LlmReasonRequest (serialized)
  │  ├─ target: Some(Agent-2)
  │  └─ conversation_id: UUID
  │
  └─ transport.publish()
       │
       ├──────── [ACP MESSAGE] ────────────→
       │
       (If Agent-2 has LlmAgent subscribed to "llm.reason.request")
       │
       handle_message()
       │
       ├─ Deserialize payload to LlmReasonRequest
       ├─ llm_client.complete(prompt)
       │
       ├─ If successful:
       │  └─ Create AcpMessage with Inform performative
       │     └─ subject: "llm.reason.response"
       │     └─ payload: LlmReasonResponse (serialized)
       │
       └─ If failed:
          └─ Create AcpMessage with Failure performative
             └─ payload: error JSON
       │
       transport.publish() ← back to Agent-1
```

### Scenario 2: Broadcast Message

```
Agent-1 (Broadcaster)        NATS/Transport          All Agents
─────────────────────        ──────────────          ──────────

broadcast_completion()
  │
  ├─ Create AcpMessage
  │  ├─ subject: "llm.reason.request"
  │  ├─ performative: Inform  ← Note: Inform not Request
  │  ├─ payload: LlmReasonRequest (serialized)
  │  ├─ target: None           ← Broadcast (no specific target)
  │  └─ conversation_id: UUID
  │
  └─ transport.publish()
       │
       ├──────── [ACP MESSAGE] ────────────→ All subscribed agents
       │                                     │
       │                              handle_message() at each
       │                                     │
       │                              Agents can optionally respond
```

## Integration Points

### With simian-base-api

```
simian-llm depends on:
  ├─ Transport trait → Used for message publishing/subscription
  ├─ AcpMessage struct → Message envelope with routing info
  ├─ AgentId type → Unique agent identifiers
  └─ Performative enum → Message intent indicators
```

### With simian-tracing

```
simian-llm uses:
  ├─ info!() → Success logs with context
  ├─ error!() → Failure logs with error details
  └─ debug!() → Detailed operation logs
```

### With simian-metrics

```
simian-llm emits:
  ├─ llm.complete.requests → Counter of completion requests
  ├─ llm.complete.errors → Counter of completion errors
  ├─ llm.chat.requests → Counter of chat requests
  ├─ llm.chat.errors → Counter of chat errors
  └─ llm.health_checks → Counter of health checks
```

## Concurrency Model

### Interior Mutability

The `LlmAgent` uses `Arc<L>` to share the LlmClient across async tasks:

```rust
pub struct LlmAgent<T: Transport, L: LlmClient> {
    llm_client: Arc<L>,  // ← Cheap clone, expensive-to-copy HTTP client
}
```

This pattern allows:
- Multiple concurrent LLM requests from the same agent
- Shared connection pooling (in LmStudioClient)
- Non-blocking clones for task spawning

### Transport Concurrency

Transport implementors must guarantee:
- Thread-safe message publishing
- Concurrent subscriptions from multiple agents
- Non-blocking publish() and subscribe() calls

### Async Task Model

All I/O operations are async:
```rust
pub async fn complete(&self, prompt: &str) -> Result<LlmResponse>
pub async fn request_completion(&self, target: &str, prompt: &str) -> Result<()>
pub async fn handle_message(&self, msg: &AcpMessage) -> Result<()>
```

This enables:
- Running on tokio runtime with efficient thread pooling
- Handling thousands of concurrent agent operations
- Graceful timeout and cancellation support

## Instrumentation Strategy

### Tracing Pattern

Every public method follows this pattern:

```rust
pub async fn request_completion(&self, target, prompt) -> Result<()> {
    info!(source = ?self.agent_id, target = ?target, prompt_len = prompt.len(),
        "Sending completion request");
    
    // ... perform work ...
    
    match result {
        Ok(_) => {
            info!(source = ?self.agent_id, "Completion published");
        }
        Err(e) => {
            error!(error = %e, "Completion failed");
        }
    }
}
```

### Metrics Pattern

Counters track all key operations:

```rust
counter!("llm.complete.requests").increment(1);

if let Err(e) = result {
    counter!("llm.complete.errors").increment(1);
}
```

## Type Safety & Trait Bounds

### Clone Requirements

```rust
LlmAgent<T: Transport + Clone, L: LlmClient + Clone>
```

- Transport is cloned to create per-agent subscriptions
- LlmClient is wrapped in Arc to avoid expensive clones
- All message types derive Clone for serialization flexibility

### Serialization

All protocol types implement Serde:

```rust
#[derive(Serialize, Deserialize)]
pub struct LlmReasonRequest { ... }
```

This enables:
- JSON serialization to ACP message payloads (via `serde_json`)
- Potential future support for other formats (bincode, MessagePack)
- Strong typing with schema validation

## Performance Characteristics

### Memory

- LlmAgent: ~200 bytes (ID + Arc pointers)
- LlmResponse: ~200 bytes + content length
- AcpMessage: ~500 bytes + payload length

### Latency

- Local LLM call (LmStudio): 500ms-5s (depends on model)
- Agent-to-agent request: Transport latency + remote LLM latency
- Message serialization: <1ms for typical messages

### Throughput

- Single LlmAgent: Limited by LLM backend (typically 1-10 req/s)
- Multiple agents: Linear scaling with independent Transport/LlmClient instances
- Message processing: 10,000+ msg/s in pure message routing

## Extensibility Points

### Custom LlmClient

Implement the trait to support new backends:

```rust
pub struct MyLlmClient;

#[async_trait]
impl LlmClient for MyLlmClient {
    async fn complete(&self, prompt: &str) -> Result<LlmResponse> { ... }
    async fn chat(&self, messages: &[ChatMessage]) -> Result<LlmResponse> { ... }
    async fn health_check(&self) -> Result<bool> { ... }
    fn model_info(&self) -> &ModelInfo { ... }
}
```

### Custom Instrumentation

Wrap LlmClient to add custom metrics:

```rust
pub struct MetricsWrapper<L: LlmClient> {
    inner: L,
    metrics_client: MetricsHandle,
}

#[async_trait]
impl<L: LlmClient> LlmClient for MetricsWrapper<L> {
    async fn complete(&self, prompt: &str) -> Result<LlmResponse> {
        let start = Instant::now();
        let result = self.inner.complete(prompt).await;
        self.metrics_client.latency("llm.complete", start.elapsed());
        result
    }
    // ...
}
```

### Custom Protocol Extensions

Extend AcpMessage handling:

```rust
impl<T, L> LlmAgent<T, L> {
    async fn handle_message(&self, msg: &AcpMessage) -> Result<()> {
        match msg.subject.as_str() {
            "llm.reason.request" => self.handle_reason_request(msg).await?,
            "llm.chat.request" => self.handle_chat_request(msg).await?,
            "custom.subject" => self.handle_custom(msg).await?,  // ← Add custom handler
            _ => {}
        }
        Ok(())
    }
}
```

## Dependencies & Compatibility

### Core Dependencies

- `async-trait` - Async trait support
- `serde/serde_json` - Serialization
- `anyhow` - Error handling
- `tracing` - Structured logging
- `metrics` - Counter/gauge emission
- `reqwest` - HTTP client (LmStudioClient)
- `simian-base-api` - ACP types
- `simian-tracing` - Tracing helpers
- `simian-metrics` - Metrics helpers

### Workspace Dependencies

All main crate dependencies are pinned in the workspace root `Cargo.toml` for consistency across the ecosystem.

---

**Next:** See [PROTOCOL.md](./PROTOCOL.md) for detailed message flow documentation.
