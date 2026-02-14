# simian-llm Examples

Practical examples demonstrating common patterns and use cases.

## Table of Contents

1. [Direct LLM Operations](#direct-llm-operations)
2. [Single-Agent Message Handling](#single-agent-message-handling)
3. [Multi-Agent Orchestration](#multi-agent-orchestration)
4. [Fallback Chains](#fallback-chains)
5. [Broadcasting Tasks](#broadcasting-tasks)
6. [Error Handling Patterns](#error-handling-patterns)
7. [Instrumentation](#instrumentation)
8. [Custom LLM Backend](#custom-llm-backend)

## Direct LLM Operations

### Simple Completion

The most basic use case: send a prompt to an LLM and get a response.

```rust
use simian_llm::{LmStudioClient, LmStudioConfig};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Configure LMStudio
    let config = LmStudioConfig::new(
        "http://localhost:1234",
        "default"
    ).with_timeout(Duration::from_secs(60));

    // Create client
    let client = LmStudioClient::new(config).await?;

    // Simple completion
    let response = client.complete(
        "What is the capital of France?"
    ).await?;

    println!("✓ {}", response.content);
    println!("  Tokens: {:?}", response.token_usage);

    Ok(())
}
```

### Multi-Turn Chat

Maintain conversation history for context.

```rust
use simian_llm::{LmStudioClient, LmStudioConfig, ChatMessage};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = LmStudioConfig::new("http://localhost:1234", "default");
    let client = LmStudioClient::new(config).await?;

    // Build conversation
    let messages = vec![
        ChatMessage::system("You are a helpful assistant"),
        ChatMessage::user("What is Rust?"),
        ChatMessage::assistant("Rust is a systems programming language..."),
        ChatMessage::user("How does ownership work?"),
    ];

    let response = client.chat(&messages).await?;
    println!("Response: {}", response.content);

    Ok(())
}
```

### Health Checks

Verify backend availability before operations.

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = LmStudioClient::new(config).await?;

    match client.health_check().await {
        Ok(true) => println!("✓ LLM backend online"),
        Ok(false) => println!("✗ LLM backend offline"),
        Err(e) => println!("✗ Error: {}", e),
    }

    Ok(())
}
```

## Single-Agent Message Handling

### Basic Listener

Create an agent that processes incoming LLM requests.

```rust
use simian_llm::LlmAgent;
use simian_nats_streams::NatsTransport;
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup
    let nats = async_nats::connect("nats://127.0.0.1:4222").await?;
    let transport = NatsTransport::new(
        nats,
        "llm-agent",
        "agents".to_string()
    ).await?;

    let client = LmStudioClient::new(config).await?;
    let agent = LlmAgent::new("llm-agent", transport.clone(), client);

    // Listen for messages
    let mut messages = transport.subscribe("llm-agent").await?;

    println!("Agent listening for requests...");

    while let Some(msg) = messages.next().await {
        println!("📨 Received: {:?}", msg.subject);

        // Automatic error handling via handle_message
        if let Err(e) = agent.handle_message(&msg).await {
            eprintln!("Error handling message: {}", e);
        }
    }

    Ok(())
}
```

### Custom Message Handling

Handle specific message types with custom logic.

```rust
use simian_base_api::transport::{Performative, AcpMessage};
use simian_llm::protocol::{LlmReasonRequest, LlmReasonResponse};

async fn process_request(agent: &LlmAgent, msg: &AcpMessage) -> anyhow::Result<()> {
    // Check if this is a reason request
    if msg.subject.contains("reason.request") {
        let request: LlmReasonRequest = serde_json::from_slice(&msg.payload)?;
        
        println!("Processing: {}", request.prompt);

        // Handle with custom logic
        let response = agent.client.complete(&request.prompt).await?;

        // Send response back
        agent.reply(&msg, &response).await?;
    }

    Ok(())
}
```

## Multi-Agent Orchestration

### Request-Response Pattern

Agent A sends request to Agent B and waits for response.

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup agents
    let nats = async_nats::connect("nats://127.0.0.1:4222").await?;

    // Agent A (requester)
    let transport_a = NatsTransport::new(
        nats.clone(),
        "requester",
        "agents".to_string()
    ).await?;
    let agent_a = LlmAgent::new("requester", transport_a.clone(), client_a);

    // Agent B (responder)
    let transport_b = NatsTransport::new(
        nats.clone(),
        "responder",
        "agents".to_string()
    ).await?;
    let agent_b = LlmAgent::new("responder", transport_b.clone(), client_b);

    // Spawn responder task
    tokio::spawn(async move {
        let mut messages = transport_b.subscribe("responder").await.unwrap();
        while let Some(msg) = messages.next().await {
            let _ = agent_b.handle_message(&msg).await;
        }
    });

    // Agent A sends request
    println!("→ Sending completion request...");
    agent_a.request_completion(
        "responder",
        "Solve: 2+2"
    ).await?;

    // Wait for response
    let mut responses = transport_a.subscribe("requester").await?;
    if let Some(msg) = responses.next().await {
        println!("← Received response from responder");
        if msg.performative == Performative::Inform {
            let resp: LlmReasonResponse = serde_json::from_slice(&msg.payload)?;
            println!("✓ Result: {}", resp.response);
        }
    }

    Ok(())
}
```

### Chained Operations

Agent A → Agent B → Agent C with result passing.

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup 3 agents
    let agent_a = create_agent("a", client_a).await?;
    let agent_b = create_agent("b", client_b).await?;
    let agent_c = create_agent("c", client_c).await?;

    // Chain: A asks B to analyze, B asks C to summarize

    // A → B: "Analyze this data"
    agent_a.request_completion("b", "Analyze: [data]").await?;

    // B listener spawned with logic:
    // On receiving from A, B calls C:
    // B → C: "Summarize this analysis"

    // C responds → B → A

    Ok(())
}
```

## Fallback Chains

### LLM Fallback

Try primary backend, fall back to secondary on failure.

```rust
use simian_llm::{LmStudioClient, LmStudioConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let primary = LmStudioClient::new(
        LmStudioConfig::new("http://primary:1234", "default")
    ).await?;

    let secondary = LmStudioClient::new(
        LmStudioConfig::new("http://secondary:1234", "default")
    ).await?;

    let prompt = "What is AI?";

    let response = match primary.complete(prompt).await {
        Ok(resp) => {
            println!("✓ Primary succeeded");
            resp
        }
        Err(e) => {
            eprintln!("✗ Primary failed: {}", e);
            println!("→ Trying secondary...");
            secondary.complete(prompt).await?
        }
    };

    println!("Result: {}", response.content);
    Ok(())
}
```

### Agent Fallback

Send request to preferred agent, escalate to backup if no response.

```rust
use tokio::time::{timeout, Duration};
use simian_llm::LlmAgent;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let agent = /* ... */;
    
    let prompt = "Process this task";
    let timeout_duration = Duration::from_secs(5);

    // Try preferred agent
    match timeout(
        timeout_duration,
        agent.request_completion("preferred-agent", prompt)
    ).await {
        Ok(Ok(())) => println!("✓ Preferred agent responded"),
        _ => {
            println!("✗ Preferred agent timeout");
            println!("→ Escalating to backup...");
            agent.request_completion("backup-agent", prompt).await?;
        }
    }

    Ok(())
}
```

## Broadcasting Tasks

### Scatter to All Agents

Send task to all agents simultaneously.

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let agent = LlmAgent::new("coordinator", transport, client);

    // Broadcast task to all agents
    println!("📡 Broadcasting analysis task to all workers...");
    agent.broadcast_completion(
        "Analyze this dataset and report your findings"
    ).await?;

    // Workers receive on their subscriptions
    // Each processes independently
    // Can respond back to coordinator

    Ok(())
}
```

### Work Distribution

Coordinator distributes work across pool.

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let agents = vec!["worker-1", "worker-2", "worker-3"];
    let tasks = vec![
        "Process batch A",
        "Process batch B", 
        "Process batch C",
    ];

    let coordinator = LlmAgent::new("coordinator", transport, client);

    // Distribute tasks round-robin
    for (i, task) in tasks.iter().enumerate() {
        let agent_id = &agents[i % agents.len()];
        coordinator.request_completion(agent_id, task).await?;
        println!("→ Sent task to {}", agent_id);
    }

    Ok(())
}
```

## Error Handling Patterns

### Graceful Degradation

Handle errors without panicking.

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = LmStudioClient::new(config).await?;

    let prompts = vec![
        "What is AI?",
        "Explain quantum computing",
        "What is machine learning?",
    ];

    for prompt in prompts {
        match client.complete(prompt).await {
            Ok(response) => {
                println!("✓ {}: {}", prompt, response.content);
            }
            Err(e) => {
                eprintln!("✗ {}: {}", prompt, e);
                println!("  → Skipping this prompt");
                continue;
            }
        }
    }

    Ok(())
}
```

### Retry with Backoff

Retry failed requests with exponential backoff.

```rust
use std::time::Duration;

async fn call_with_retry(
    client: &LmStudioClient,
    prompt: &str,
    max_retries: u32,
) -> anyhow::Result<LlmResponse> {
    for attempt in 0..max_retries {
        match client.complete(prompt).await {
            Ok(response) => return Ok(response),
            Err(e) if attempt < max_retries - 1 => {
                let backoff = Duration::from_millis(2_u64.pow(attempt) * 100);
                eprintln!(
                    "Attempt {} failed: {}. Retrying in {:?}...",
                    attempt + 1, e, backoff
                );
                tokio::time::sleep(backoff).await;
            }
            Err(e) => return Err(e.into()),
        }
    }
    unreachable!()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = LmStudioClient::new(config).await?;
    let response = call_with_retry(&client, "What is Rust?", 3).await?;
    println!("{}", response.content);
    Ok(())
}
```

### Timeout Protection

Prevent hanging on slow backends.

```rust
use tokio::time::{timeout, Duration};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = LmStudioClient::new(
        LmStudioConfig::new("http://localhost:1234", "default")
            .with_timeout(Duration::from_secs(30))
    ).await?;

    // Timeout wraps the HTTP call
    match timeout(
        Duration::from_secs(35),  // Outer timeout
        client.complete("Long prompt")
    ).await {
        Ok(Ok(response)) => println!("✓ {}", response.content),
        Ok(Err(e)) => eprintln!("✗ LLM error: {}", e),
        Err(_) => eprintln!("✗ Request timeout"),
    }

    Ok(())
}
```

## Instrumentation

### Structured Logging

Log important events with context.

```rust
use tracing::{info, error, warn, span, Level};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let span = span!(Level::DEBUG, "llm_operation");
    let _guard = span.enter();

    let client = LmStudioClient::new(config).await?;

    match client.complete("Analyze this").await {
        Ok(response) => {
            info!(
                tokens = response.token_usage.as_ref().map(|u| u.total),
                "✓ Completion succeeded"
            );
        }
        Err(e) => {
            error!(error = %e, "✗ Completion failed");
        }
    }

    Ok(())
}
```

### Metrics Collection

Track performance and errors.

```rust
use metrics::{counter, histogram};
use std::time::Instant;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = LmStudioClient::new(config).await?;

    let start = Instant::now();
    
    match client.complete("Prompt").await {
        Ok(response) => {
            let duration = start.elapsed().as_secs_f64();
            counter!("llm.complete.success").increment(1);
            histogram!("llm.complete.duration").record(duration);
        }
        Err(e) => {
            counter!("llm.complete.error").increment(1);
        }
    }

    Ok(())
}
```

## Custom LLM Backend

### Implementing a New Backend

See [BACKEND_GUIDE.md](./BACKEND_GUIDE.md) for detailed instructions.

Quick example adding OpenAI support:

```rust
use simian_llm::{LlmClient, LlmResponse, ChatMessage, ModelInfo};
use async_trait::async_trait;

#[derive(Clone)]
pub struct OpenAiClient {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

#[async_trait]
impl LlmClient for OpenAiClient {
    async fn complete(&self, prompt: &str) -> anyhow::Result<LlmResponse> {
        let response = self.client
            .post("https://api.openai.com/v1/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "model": self.model,
                "prompt": prompt,
                "max_tokens": 1024,
            }))
            .send()
            .await?
            .json::<OpenAiResponse>()
            .await?;

        Ok(LlmResponse {
            content: response.choices[0].text.clone(),
            token_usage: None,
            model: self.model.clone(),
        })
    }

    async fn chat(&self, messages: &[ChatMessage]) -> anyhow::Result<LlmResponse> {
        // Implementation similar to complete()
        todo!()
    }

    async fn health_check(&self) -> anyhow::Result<bool> {
        Ok(self.client
            .get("https://api.openai.com/v1/models")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?
            .status()
            .is_success())
    }

    fn model_info(&self) -> &ModelInfo {
        todo!()
    }
}
```

Then use it:

```rust
let client = OpenAiClient::new("sk-...", "gpt-4");
let response = client.complete("Explain quantum computing").await?;
println!("{}", response.content);
```

## More Examples

- **[agent_llm_chat.rs](./examples/agent_llm_chat.rs)** - Multi-agent communication
- **[llm_test.rs](./examples/llm_test.rs)** - Direct LLM testing with real backend
- **[debug_lm.rs](./examples/debug_lm.rs)** - API debugging utilities

Run examples:

```bash
cargo run --example agent_llm_chat -p simian-llm --release
cargo run --example llm_test -p simian-llm --release
```

## See Also

- [ARCHITECTURE.md](./ARCHITECTURE.md) - System design
- [PROTOCOL.md](./PROTOCOL.md) - Message format
- [INTEGRATION_GUIDE.md](./INTEGRATION_GUIDE.md) - Integration patterns
- [BACKEND_GUIDE.md](./BACKEND_GUIDE.md) - Adding new backends
