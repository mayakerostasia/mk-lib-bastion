# simian-llm

Maya: this is where the magic happens of hooking these Reactors and streams into Large Language models

LLM integration library for the simian-bastion agent framework. Provides core traits, types, and an ACP message protocol for routing LLM requests and responses between agents.

## What's Inside

| Module | Description |
|--------|-------------|
| `client` | `LlmClient` trait — implement this for any LLM backend (LMStudio, OpenAI, Ollama, etc.) |
| `types` | `ChatMessage`, `ChatRole`, `ModelInfo`, `TokenUsage`, `LlmResponse` |
| `error` | `LlmError` enum for connection, timeout, rate-limit, and parse failures |
| `protocol` | ACP message payloads (`LlmReasonRequest/Response`, `LlmChatRequest/Response`) and subject constants |

## Prerequisites

- **Rust** toolchain (1.75+ recommended) — [install via rustup](https://rustup.rs/)
- **NATS server** running and reachable from the machine

### Installing NATS

Pick whichever method suits your OS:

```bash
# macOS
brew install nats-server

# Linux (amd64 binary)
curl -L https://github.com/nats-io/nats-server/releases/latest/download/nats-server-v2.10.24-linux-amd64.tar.gz \
  | tar xz && sudo mv nats-server-v2.10.24-linux-amd64/nats-server /usr/local/bin/

# Windows (scoop)
scoop install nats-server

# Docker (any OS)
docker run -d --name nats -p 4222:4222 nats:latest
```

Start it:

```bash
nats-server          # foreground, listens on 0.0.0.0:4222
nats-server -D       # with debug logging
```

Verify it's up:

```bash
# Linux / macOS
nc -zv 127.0.0.1 4222

# Windows PowerShell
Test-NetConnection -ComputerName 127.0.0.1 -Port 4222
```

## Clone & Build

```bash
git clone <repo-url> mk-lib-bastion
cd mk-lib-bastion

# Build just this crate (and its dependencies)
cargo build -p simian-llm

# Build the entire workspace
cargo build --workspace
```

## Running the Tests

### Unit tests (no external services needed)

```bash
cargo test -p simian-llm --lib
```

These cover serde round-trips for all types and protocol structures.

### Integration test (requires NATS)

The integration test spins up two `SimianAgent` instances that exchange `LlmReasonRequest` / `LlmReasonResponse` messages over a real NATS connection.

```bash
# Make sure nats-server is running on 127.0.0.1:4222, then:
cargo test -p simian-llm --test nats_integration -- --nocapture
```

**If NATS is on a different host or port**, edit the URL in `tests/nats_integration.rs`:

```rust
NatsTransport::new("nats://<host>:<port>", "llmtest")
```

## Usage in Your Own Crate

Add the dependency in your `Cargo.toml`:

```toml
[dependencies]
simian-llm = { path = "../simian-llm" }
simian-base-api = { path = "../simian-base-api" }
simian-nats-streams = { path = "../simian-nats-streams" }
```

### Sending an LLM request between agents

```rust
use bytes::Bytes;
use futures::StreamExt;
use simian_base_api::agent::SimianAgent;
use simian_base_api::transport::{AgentId, Performative};
use simian_llm::{LlmReasonRequest, LlmReasonResponse};
use simian_llm::protocol::subjects;
use simian_nats_streams::transport::NatsTransport;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let transport = NatsTransport::new("nats://127.0.0.1:4222", "agents").await?;
    let agent = SimianAgent::new(AgentId::new("my-agent"), transport);

    // Build and send a reason request
    let request = LlmReasonRequest {
        prompt: "Summarize this document.".into(),
        context: None,
    };
    let payload = Bytes::from(serde_json::to_vec(&request)?);
    agent.request(
        AgentId::new("llm-agent"),
        subjects::LLM_REASON_REQUEST,
        payload,
    ).await?;

    // Listen for the response
    let mut inbox = agent.listen().await?;
    if let Some(msg) = inbox.next().await {
        let resp: LlmReasonResponse = serde_json::from_slice(&msg.payload)?;
        println!("LLM says: {}", resp.response);
    }

    Ok(())
}
```

### Implementing the `LlmClient` trait

To plug in your own LLM backend, implement `LlmClient`:

```rust
use simian_llm::{LlmClient, ChatMessage, LlmResponse, ModelInfo};
use anyhow::Result;
use async_trait::async_trait;

#[derive(Debug, Clone)]
struct MyLlmBackend {
    model_info: ModelInfo,
}

#[async_trait]
impl LlmClient for MyLlmBackend {
    async fn complete(&self, prompt: &str) -> Result<LlmResponse> {
        // Call your LLM HTTP API here
        todo!()
    }

    async fn chat(&self, messages: &[ChatMessage]) -> Result<LlmResponse> {
        todo!()
    }

    async fn health_check(&self) -> Result<bool> {
        // Ping your LLM server
        todo!()
    }

    fn model_info(&self) -> &ModelInfo {
        &self.model_info
    }
}
```

## ACP Subject Patterns

| Constant | Value | Direction |
|----------|-------|-----------|
| `LLM_REASON_REQUEST` | `llm.reason.request` | requester → llm-agent |
| `LLM_REASON_RESPONSE` | `llm.reason.response` | llm-agent → requester |
| `LLM_CHAT_REQUEST` | `llm.chat.request` | requester → llm-agent |
| `LLM_CHAT_RESPONSE` | `llm.chat.response` | llm-agent → requester |

## Troubleshooting

| Problem | Fix |
|---------|-----|
| `Failed to connect transport` | NATS isn't running or is on a different address. Start it with `nats-server` and check the port. |
| `timed out waiting for message` | Subscription may not have propagated. The test uses a 200ms delay — increase it on slow networks. |
| `cargo build` fails on `bytes::Bytes` serde | Ensure the workspace `Cargo.toml` has `bytes = { version = "1.6.0", features = ["serde"] }`. |
