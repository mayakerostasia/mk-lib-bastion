# Development Guide for simian-llm

Guidelines for contributing to the simian-llm crate.

## Table of Contents

1. [Setup](#setup)
2. [Architecture](#architecture)
3. [Code Organization](#code-organization)
4. [Writing Code](#writing-code)
5. [Testing](#testing)
6. [Documentation](#documentation)
7. [Submitting Changes](#submitting-changes)
8. [Performance Considerations](#performance-considerations)

## Setup

### Prerequisites

- Rust 1.75+
- Cargo
- Git
- NATS server (for integration tests)
- LMStudio (for testing LMStudio client)

### Initial Setup

```bash
# Clone repository
git clone <repo-url> mk-lib-bastion
cd mk-lib-bastion

# Build entire workspace
cargo build --workspace

# Run tests
cargo test -p simian-llm

# Check formatting
cargo fmt --check -p simian-llm

# Run clippy
cargo clippy -p simian-llm
```

## Architecture

### Core Components

**LlmClient Trait** (`src/client.rs`)
- Abstract interface for LLM backends
- Methods: `complete()`, `chat()`, `health_check()`, `model_info()`
- Implementations: `LmStudioClient`, (future: `OpenAiClient`, etc.)

**LlmAgent** (`src/agent.rs`)
- Generic wrapper combining Transport + LlmClient
- Handles ACP message routing
- Automatic response generation

**Protocol** (`src/protocol.rs`)
- ACP message payloads
- Subject constants
- Request/response types

**LmStudio Client** (`src/lmstudio/`)
- HTTP client for OpenAI-compatible API
- Bearer token authentication
- Configurable timeouts, model, token limits

**Types** (`src/types.rs`)
- `ChatMessage`, `ChatRole`
- `LlmResponse`, `TokenUsage`
- `ModelInfo`

**Error Types** (`src/error.rs`)
- `LlmError` enum
- Implements `std::error::Error`

### Dependencies

All pinned in workspace root `Cargo.toml`:

```toml
[workspace.dependencies]
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
reqwest = { version = "0.11", features = ["json"] }
tokio = { version = "1", features = ["rt-multi-thread"] }
```

## Code Organization

```
simian-llm/
├── src/
│   ├── lib.rs              # Crate root, public exports
│   ├── client.rs           # LlmClient trait definition
│   ├── agent.rs            # LlmAgent<T, L> implementation
│   ├── types.rs            # Shared types
│   ├── error.rs            # Error types
│   ├── protocol.rs         # ACP message structures
│   └── lmstudio/
│       ├── mod.rs          # Module root
│       ├── config.rs       # LmStudioConfig
│       ├── client.rs       # LmStudioClient implementation
│       └── api.rs          # HTTP API types
├── tests/
│   └── nats_integration.rs # NATS e2e test
├── examples/
│   ├── llm_test.rs         # Direct LLM testing
│   ├── agent_llm_chat.rs   # Multi-agent example
│   └── debug_lm.rs         # API debugging
├── Cargo.toml
├── README.md
├── ARCHITECTURE.md
├── PROTOCOL.md
├── INTEGRATION_GUIDE.md
├── BACKEND_GUIDE.md
├── EXAMPLES.md
├── TROUBLESHOOTING.md
└── DEVELOPMENT.md (this file)
```

## Writing Code

### Code Style

- **Line length**: 100 characters (soft limit)
- **Formatting**: Run `cargo fmt` before committing
- **Naming**: 
  - Types: `PascalCase`
  - Functions: `snake_case`
  - Constants: `SCREAMING_SNAKE_CASE`
  - Traits: `PascalCase` ending with `-able` or `-or`

### Module Organization

Keep modules focused on single responsibility:

```rust
// ✓ Good: Clear responsibility
mod lmstudio {
    pub mod config;    // Configuration
    pub mod client;    // HTTP client impl
    pub mod api;       // API types
}

// ✗ Bad: Mixed concerns
pub struct LmStudioConfig { /* ... */ }
pub struct LmStudioClient { /* ... */ }
pub struct ApiRequest { /* ... */ }
// All in same file
```

### Trait Implementations

Always use `async_trait` for async trait methods:

```rust
use async_trait::async_trait;

#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn complete(&self, prompt: &str) -> Result<LlmResponse>;
    
    fn model_info(&self) -> &ModelInfo;
}
```

Derive `Clone` and `Debug` for client structs:

```rust
#[derive(Clone)]
pub struct MyClient {
    client: reqwest::Client,
    config: Config,
}

impl Debug for MyClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MyClient")
            .field("config", &self.config)
            .finish()
    }
}
```

### Error Handling

Prefer `anyhow::Result<T>` for public APIs:

```rust
pub async fn complete(&self, prompt: &str) -> anyhow::Result<LlmResponse> {
    // ...
}
```

Log errors with context, don't panic:

```rust
// ✓ Good
tracing::error!(error = %e, "Failed to parse response");

// ✗ Bad
panic!("Failed to parse: {:?}", e);
```

### Instrumentation

Add tracing and metrics to all public methods:

```rust
pub async fn complete(&self, prompt: &str) -> anyhow::Result<LlmResponse> {
    use tracing::info;
    use metrics::counter;
    
    counter!("llm.complete.requests").increment(1);
    
    info!(
        prompt_len = prompt.len(),
        model = self.model_info().name,
        "Starting completion"
    );
    
    match self.call_api(prompt).await {
        Ok(response) => {
            info!(
                tokens = response.token_usage.as_ref().map(|u| u.total),
                "Completion succeeded"
            );
            Ok(response)
        }
        Err(e) => {
            counter!("llm.complete.errors").increment(1);
            tracing::error!(error = %e, "Completion failed");
            Err(e)
        }
    }
}
```

### Comments

Only comment why, not what:

```rust
// ✓ Good: Explains the why
// Use Bearer token format required by OpenAI-compatible API
request = request.header(
    "Authorization",
    format!("Bearer {}", self.api_token)
);

// ✗ Bad: Obvious from code
// Add authorization header
request = request.header("Authorization", token);
```

## Testing

### Unit Tests

Place near implementation in `#[cfg(test)]` modules:

```rust
// In same file as implementation
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = LmStudioConfig::new("http://localhost:1234", "default")
            .with_timeout(Duration::from_secs(60));
        
        assert_eq!(config.base_url, "http://localhost:1234");
        assert_eq!(config.model, "default");
    }

    #[tokio::test]
    async fn test_complete() {
        let client = MockLlmClient::new();
        let response = client.complete("test").await.unwrap();
        
        assert!(!response.content.is_empty());
    }
}
```

### Integration Tests

Place in `tests/` directory for E2E scenarios:

```rust
// tests/nats_integration.rs
#[tokio::test]
async fn test_agent_communication() {
    let transport = NatsTransport::new(nats, "test", "agents").await.unwrap();
    let agent = LlmAgent::new("test-agent", transport, client);
    
    // E2E test
}
```

### Mock Clients

Create mock implementations for testing:

```rust
#[cfg(test)]
struct MockLlmClient {
    responses: Vec<LlmResponse>,
}

#[cfg(test)]
#[async_trait]
impl LlmClient for MockLlmClient {
    async fn complete(&self, _prompt: &str) -> anyhow::Result<LlmResponse> {
        Ok(self.responses[0].clone())
    }
    
    // ...
}
```

### Test Coverage

Aim for >80% coverage:

```bash
# Generate coverage report (requires tarpaulin)
cargo tarpaulin -p simian-llm --out Html
```

### Running Tests

```bash
# All tests
cargo test -p simian-llm

# Unit tests only
cargo test -p simian-llm --lib

# Integration tests
cargo test -p simian-llm --test '*'

# With output
cargo test -p simian-llm -- --nocapture

# Specific test
cargo test -p simian-llm complete

# Ignored tests (e.g., requires LMStudio)
cargo test -p simian-llm -- --ignored
```

## Documentation

### Doc Comments

Add doc comments to all public items:

```rust
/// Completes a prompt using the LLM backend.
///
/// # Arguments
///
/// * `prompt` - The prompt text to complete
///
/// # Returns
///
/// Returns `LlmResponse` with generated text and token usage.
///
/// # Errors
///
/// Returns `LlmError` if:
/// - The LLM backend is unreachable
/// - The request times out
/// - The response cannot be parsed
///
/// # Examples
///
/// ```
/// let client = LmStudioClient::new(config).await?;
/// let response = client.complete("What is Rust?").await?;
/// println!("{}", response.content);
/// ```
pub async fn complete(&self, prompt: &str) -> anyhow::Result<LlmResponse> {
    // ...
}
```

### Architecture Documentation

Update [ARCHITECTURE.md](./ARCHITECTURE.md) when:
- Adding new components
- Changing trait design
- Modifying concurrency model
- Updating dependency graph

### Example Documentation

Add examples to [EXAMPLES.md](./EXAMPLES.md) when:
- Implementing new backend
- Adding new public API
- Creating new usage pattern

### Troubleshooting Guide

Update [TROUBLESHOOTING.md](./TROUBLESHOOTING.md) for:
- Common errors
- Platform-specific issues
- Performance gotchas
- Known limitations

## Submitting Changes

### Before Committing

```bash
# Format code
cargo fmt -p simian-llm

# Check for issues
cargo clippy -p simian-llm

# Run tests
cargo test -p simian-llm

# Build successfully
cargo build -p simian-llm
```

### Commit Message Format

```
<type>: <subject>

<body>

<footer>
```

Examples:
```
feat: Add OpenAI backend implementation

Implements LlmClient for OpenAI API with full token counting and streaming support.

Closes #123
```

```
fix: Handle LMStudio timeout gracefully

Previously timed out requests would panic. Now returns LlmError::Timeout with proper logging.

Fixes #456
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `test`: Test additions
- `refactor`: Code reorganization
- `perf`: Performance improvements
- `chore`: Dependency updates, etc.

### Pull Request Checklist

- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] Code formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] All tests pass (`cargo test`)
- [ ] Commit messages follow convention
- [ ] No unnecessary files committed

## Performance Considerations

### Memory

- **LlmAgent**: ~200 bytes (small fixed size)
- **LmStudioClient**: ~2KB (HTTP client + config)
- **Message buffers**: Depends on payload size

### Connection Pooling

LmStudioClient reuses HTTP connections via `reqwest::Client`:

```rust
// ✓ Good: Reuse client
let client = LmStudioClient::new(config).await?;
for prompt in prompts {
    let response = client.complete(prompt).await?;
}

// ✗ Bad: Creates new connection each time
for prompt in prompts {
    let client = LmStudioClient::new(config).await?;
    let response = client.complete(prompt).await?;
}
```

### Async Efficiency

Use async-await patterns to avoid blocking:

```rust
// ✓ Good: Concurrent requests
let f1 = agent.request_completion("agent-1", "prompt-1");
let f2 = agent.request_completion("agent-2", "prompt-2");
let (r1, r2) = tokio::join!(f1, f2);

// ✗ Bad: Sequential requests
agent.request_completion("agent-1", "prompt-1").await?;
agent.request_completion("agent-2", "prompt-2").await?;
```

### Serialization Overhead

- JSON serialization/deserialization: ~1-2ms for typical payloads
- Consider message batching if throughput is critical

## Benchmarking

Create benchmarks for performance-sensitive code:

```rust
// benches/llm_client.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_complete(c: &mut Criterion) {
    c.bench_function("llm_complete_1k", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| async {
                let client = create_client();
                client.complete(black_box("prompt")).await
            });
    });
}

criterion_group!(benches, bench_complete);
criterion_main!(benches);
```

Run:
```bash
cargo bench -p simian-llm
```

## Adding a New LLM Backend

1. **Create module** in `src/{backend_name}/`
2. **Implement LlmClient trait**
3. **Add tests** with real API and mocks
4. **Add to exports** in `lib.rs`
5. **Document** in BACKEND_GUIDE.md
6. **Add example** in `examples/`

See [BACKEND_GUIDE.md](./BACKEND_GUIDE.md) for complete walkthrough.

## Debugging

### Enable Verbose Logging

```bash
export RUST_LOG=debug
cargo run --example agent_llm_chat
```

### Print Debug Info

```rust
use tracing::debug;

debug!(
    client = ?client,
    config = ?config,
    "Client configuration"
);
```

### Interactive Debugging

```bash
# With GDB/LLDB
rust-gdb --args target/debug/binary arg1 arg2
```

### Profiling

```bash
# CPU profile
perf record ./target/release/binary
perf report

# Memory profile
valgrind --leak-check=full ./target/release/binary
```

## Common Patterns

### Builder Pattern for Configuration

```rust
pub struct LmStudioConfig {
    pub base_url: String,
    pub model: String,
    pub api_token: Option<String>,
    pub timeout: Duration,
}

impl LmStudioConfig {
    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            model: model.into(),
            api_token: None,
            timeout: Duration::from_secs(30),
        }
    }

    pub fn with_api_token(mut self, token: impl Into<String>) -> Self {
        self.api_token = Some(token.into());
        self
    }
}
```

### Generic Over Transport and Client

```rust
pub struct LlmAgent<T: Transport, L: LlmClient> {
    agent_id: String,
    transport: T,
    client: Arc<L>,
}

impl<T: Transport, L: LlmClient> LlmAgent<T, L> {
    pub fn new(agent_id: impl Into<String>, transport: T, client: L) -> Self {
        Self {
            agent_id: agent_id.into(),
            transport,
            client: Arc::new(client),
        }
    }
}
```

## CI/CD Integration

The repository includes GitHub Actions workflows. Ensure:

- All tests pass locally before pushing
- Code is formatted and linted
- No new clippy warnings
- Documentation is updated

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Async Rust](https://rust-lang.github.io/async-book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Project Conventions](../DEVELOPMENT.md)

## Questions?

Check:
- [README.md](./README.md) - Quick reference
- [ARCHITECTURE.md](./ARCHITECTURE.md) - System design
- [BACKEND_GUIDE.md](./BACKEND_GUIDE.md) - Backend implementation
- [TROUBLESHOOTING.md](./TROUBLESHOOTING.md) - Common issues
