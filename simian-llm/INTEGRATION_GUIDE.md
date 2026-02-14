# simian-llm Integration Guide

A practical guide to integrating simian-llm into your agent-based systems.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Basic Usage](#basic-usage)
3. [LMStudio Integration](#lmstudio-integration)
4. [Agent-to-Agent Communication](#agent-to-agent-communication)
5. [Error Handling](#error-handling)
6. [Configuration Best Practices](#configuration-best-practices)
7. [Testing](#testing)

## Quick Start

### 1. Add to Cargo.toml

```toml
[dependencies]
simian-llm = { path = "../../simian-llm" }
simian-base-api = { path = "../../simian-base-api" }
simian-nats-streams = { path = "../../simian-nats-streams" }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

### 2. Create a Simple Agent

```rust
use simian_llm::{LmStudioClient, LmStudioConfig, LlmAgent};
use simian_nats_streams::NatsTransport;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Connect to NATS
    let nats_client = async_nats::connect("nats://127.0.0.1:4222").await?;
    let transport = NatsTransport::new(
        nats_client,
        "my-agent",
        "agents".to_string()
    ).await?;

    // 2. Create LLM client
    let config = LmStudioConfig::new("http://localhost:1234", "default")
        .with_timeout(Duration::from_secs(60));
    let llm = LmStudioClient::new(config).await?;

    // 3. Create LLM agent
    let agent = LlmAgent::new("my-agent", transport, llm);

    // 4. Use the agent
    agent.request_completion("other-agent", "What is Rust?").await?;

    Ok(())
}
```

## Basic Usage

### Direct LLM Calls

#### Completion (Text Generation)

```rust
use simian_llm::{LmStudioClient, LmStudioConfig};
use std::time::Duration;

let config = LmStudioConfig::new("http://localhost:1234", "default")
    .with_temperature(0.7)
    .with_max_tokens(256)
    .with_timeout(Duration::from_secs(30));

let client = LmStudioClient::new(config).await?;

// Simple completion
let response = client.complete("Explain machine learning in one sentence.").await?;
println!("Response: {}", response.content);

// Access metadata
if let Some(usage) = response.usage {
    println!("Tokens: {} / {} / {}",
        usage.prompt_tokens.unwrap_or(0),
        usage.completion_tokens.unwrap_or(0),
        usage.total_tokens.unwrap_or(0)
    );
}
```

#### Chat (Multi-Turn Conversation)

```rust
use simian_llm::{ChatMessage, LmStudioClient, LmStudioConfig};

let client = LmStudioClient::new(config).await?;

// Build conversation
let messages = vec![
    ChatMessage::system("You are a helpful Rust expert."),
    ChatMessage::user("What is the borrow checker?"),
];

// Send chat request
let response = client.chat(&messages).await?;
println!("Assistant: {}", response.content);
```

#### Health Checks

```rust
if client.health_check().await? {
    println!("✓ LLM backend is operational");
} else {
    println!("⚠ LLM backend is not responding");
}
```

### Agent-to-Agent Communication

#### Sending Requests

```rust
let agent = LlmAgent::new("agent-1", transport, client);

// Send completion request to specific agent
agent.request_completion("agent-2", "What is AI?").await?;

// Send chat request to specific agent
let messages = vec![
    ChatMessage::user("Explain neural networks."),
];
agent.request_chat("agent-2", &messages).await?;

// Broadcast request to all agents
agent.broadcast_completion("Solve this optimization problem: ...").await?;
```

#### Handling Incoming Requests

In a real application, agents should subscribe and process messages:

```rust
use futures::StreamExt;

// Subscribe to incoming messages
let mut messages = transport.subscribe(&agent_id).await?;

while let Some(msg) = messages.next().await {
    // Process message
    if let Err(e) = agent.handle_message(&msg).await {
        eprintln!("Error processing message: {}", e);
    }
}
```

## LMStudio Integration

### Installation & Setup

1. **Download LMStudio** from https://lmstudio.ai
2. **Start LMStudio** and download a model
3. **Start the local API server**:
   - Settings → Local Server
   - Default: http://localhost:1234

### Configuration Examples

#### Basic Configuration

```rust
let config = LmStudioConfig::new("http://localhost:1234", "default");
let client = LmStudioClient::new(config).await?;
```

#### With Authentication

```rust
let config = LmStudioConfig::new("http://192.168.1.7:1234", "default")
    .with_api_token("sk-lm-your-token-here");
let client = LmStudioClient::new(config).await?;
```

#### Production-Ready Configuration

```rust
let config = LmStudioConfig::new(
    std::env::var("LM_API_URL").unwrap_or_else(|_| "http://localhost:1234".to_string()),
    std::env::var("LM_MODEL").unwrap_or_else(|_| "default".to_string())
)
.with_api_token(std::env::var("LM_API_TOKEN").ok())
.with_temperature(0.7)
.with_max_tokens(2048)
.with_timeout(Duration::from_secs(120));

let client = LmStudioClient::new(config).await?;
```

### Available Models

LMStudio supports OpenAI-compatible models. Popular choices:

- `gemma-3-27b-it-abliterated` - Fast, good general-purpose
- `mistral-7b-instruct-v0.3` - Efficient, good for edge
- `llama-2-13b-chat` - Versatile, well-tested
- `neural-chat-7b-v3` - Optimized for chat
- Others via HuggingFace integration

### Testing LMStudio Connection

```rust
use simian_llm::{LmStudioClient, LmStudioConfig};

async fn test_connection(url: &str, token: Option<&str>) -> anyhow::Result<()> {
    let mut config = LmStudioConfig::new(url, "default");
    if let Some(t) = token {
        config = config.with_api_token(t);
    }
    
    let client = LmStudioClient::new(config).await?;
    
    if client.health_check().await? {
        println!("✓ Connection successful");
        let info = client.model_info();
        println!("  Model: {}", info.name);
        println!("  Provider: {}", info.provider);
    } else {
        println!("⚠ Connection failed");
    }
    
    Ok(())
}
```

## Agent-to-Agent Communication

### Pattern 1: Publish-Subscribe

```rust
async fn agent_handler(agent: LlmAgent<T, L>) -> anyhow::Result<()> {
    // Subscribe to messages
    let mut subscription = agent.transport.subscribe(agent.agent_id()).await?;
    
    // Handle incoming messages indefinitely
    while let Some(msg) = subscription.next().await {
        match agent.handle_message(&msg).await {
            Ok(_) => tracing::debug!("Message processed"),
            Err(e) => tracing::error!("Error: {}", e),
        }
    }
    
    Ok(())
}
```

### Pattern 2: Request Correlation

```rust
use std::collections::HashMap;
use uuid::Uuid;

struct RequestTracker {
    pending: HashMap<Uuid, String>, // conversation_id -> agent_id
}

// When sending request, track it
let conv_id = Uuid::new_v4();
tracker.pending.insert(conv_id, "agent-2".to_string());
agent.request_completion("agent-2", "...").await?;

// When receiving response, correlate it
if let Some(requester) = tracker.pending.get(&msg.conversation_id) {
    println!("Response from {} for conversation {}", requester, msg.conversation_id);
    tracker.pending.remove(&msg.conversation_id);
}
```

### Pattern 3: Broadcast Processing

```rust
// Broadcast a task to all agents
agent.broadcast_completion("Task: Analyze this data...").await?;

// Agents independently process and optionally respond
// (No direct correlation needed)
```

## Error Handling

### Handle Connection Errors

```rust
match LmStudioClient::new(config).await {
    Ok(client) => println!("✓ Connected"),
    Err(e) => {
        eprintln!("Connection failed: {}", e);
        // Fallback: use mock client, queue requests, or fail fast
    }
}
```

### Handle Request Errors

```rust
match client.complete(prompt).await {
    Ok(response) => println!("✓ {}", response.content),
    Err(e) => match e.kind() {
        ErrorKind::Timeout => eprintln!("Request timed out"),
        ErrorKind::Connection => eprintln!("Connection lost"),
        _ => eprintln!("Other error: {}", e),
    }
}
```

### Graceful Degradation

```rust
async fn get_completion(
    client: &LmStudioClient,
    prompt: &str,
    fallback: &str,
) -> String {
    match client.complete(prompt).await {
        Ok(response) => response.content,
        Err(_) => fallback.to_string(), // Use fallback on error
    }
}
```

## Configuration Best Practices

### Environment Variables

```rust
// Create a config struct
pub struct LlmConfig {
    pub api_url: String,
    pub api_token: Option<String>,
    pub model: String,
    pub timeout_secs: u64,
    pub max_tokens: u32,
    pub temperature: f32,
}

impl LlmConfig {
    pub fn from_env() -> Self {
        Self {
            api_url: std::env::var("LM_API_URL")
                .unwrap_or_else(|_| "http://localhost:1234".to_string()),
            api_token: std::env::var("LM_API_TOKEN").ok(),
            model: std::env::var("LM_MODEL")
                .unwrap_or_else(|_| "default".to_string()),
            timeout_secs: std::env::var("LM_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(60),
            max_tokens: std::env::var("LM_MAX_TOKENS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1024),
            temperature: std::env::var("LM_TEMPERATURE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.7),
        }
    }

    pub fn to_llm_config(&self) -> LmStudioConfig {
        let mut config = LmStudioConfig::new(&self.api_url, &self.model);
        if let Some(token) = &self.api_token {
            config = config.with_api_token(token);
        }
        config
            .with_timeout(Duration::from_secs(self.timeout_secs))
            .with_max_tokens(self.max_tokens)
            .with_temperature(self.temperature)
    }
}
```

### Config Files

```toml
# config/llm.toml
[development]
api_url = "http://localhost:1234"
model = "default"
timeout_secs = 30
max_tokens = 512
temperature = 0.7

[production]
api_url = "http://llm-server:1234"
model = "production-model"
timeout_secs = 120
max_tokens = 2048
temperature = 0.5
```

## Testing

### Unit Tests with Mock Client

```rust
use simian_llm::{LlmClient, LlmResponse, ChatMessage};
use async_trait::async_trait;
use anyhow::Result;

#[derive(Clone)]
struct MockLlmClient;

#[async_trait]
impl LlmClient for MockLlmClient {
    async fn complete(&self, prompt: &str) -> Result<LlmResponse> {
        Ok(LlmResponse {
            content: format!("Mock response to: {}", prompt),
            model: "mock".to_string(),
            usage: None,
        })
    }

    async fn chat(&self, messages: &[ChatMessage]) -> Result<LlmResponse> {
        Ok(LlmResponse {
            content: format!("Mock chat response to {} messages", messages.len()),
            model: "mock".to_string(),
            usage: None,
        })
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }

    fn model_info(&self) -> &ModelInfo {
        // Return a static ModelInfo
    }
}

#[tokio::test]
async fn test_agent_request() {
    let transport = MockTransport::new();
    let llm = MockLlmClient;
    let agent = LlmAgent::new("test-agent", transport.clone(), llm);

    agent
        .request_completion("other-agent", "test")
        .await
        .unwrap();

    assert_eq!(transport.message_count(), 1);
}
```

### Integration Tests with Real LMStudio

```rust
#[tokio::test]
#[ignore] // Only run with `cargo test -- --ignored`
async fn test_real_lmstudio() -> anyhow::Result<()> {
    let config = LmStudioConfig::new("http://localhost:1234", "default")
        .with_timeout(Duration::from_secs(30));
    
    let client = LmStudioClient::new(config).await?;
    
    let response = client.complete("Hello").await?;
    assert!(!response.content.is_empty());
    
    Ok(())
}
```

### Testing with Different Transports

```rust
use simian_base_api::transport::Transport;

async fn test_with_transport<T: Transport + Clone>(transport: T) {
    let client = MockLlmClient;
    let agent = LlmAgent::new("test", transport, client);
    
    // Test agent behavior
    agent.request_completion("other", "test").await.unwrap();
}

#[tokio::test]
async fn test_with_demo_transport() {
    let transport = DemoTransport::new();
    test_with_transport(transport).await;
}

#[tokio::test]
async fn test_with_nats_transport() {
    let nats = async_nats::connect("nats://127.0.0.1:4222").await.unwrap();
    let transport = NatsTransport::new(nats, "test", "agents".to_string())
        .await
        .unwrap();
    test_with_transport(transport).await;
}
```

---

**Next:** See [BACKEND_GUIDE.md](./BACKEND_GUIDE.md) to implement support for other LLM backends.
