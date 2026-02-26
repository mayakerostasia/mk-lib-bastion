# Simian-Reactor Examples

This directory contains examples demonstrating how to use the `simian-reactor` library.

## Overview

`simian-reactor` provides:
- **Frame Protocol**: Zero-copy serialization with `rkyv` for efficient message passing
- **Tower Integration**: Build composable services with middleware (rate limiting, buffering, timeouts)
- **MakoReactor**: Service wrapper with dynamic function registration and request management

## Examples

### 1. Frame Serialization (`frame_serialization.rs`)

Demonstrates the Frame protocol and serialization capabilities:

```bash
cargo run --example frame_serialization
```

**What you'll learn:**
- Creating different Frame variants (Msg, Json, Exec, Bytes, etc.)
- Encoding/decoding frames with rkyv
- Using `From<Frame> for Bytes` trait conversions
- Working with JSON data in frames
- SendBox for message routing
- Performance characteristics of serialization

**Key concepts:**
```rust
// Create and serialize frames
let frame = Frame::json(json!({"key": "value"}));
let encoded = frame.encode()?;
let decoded = Frame::decode(&encoded)?;

// Use From trait
let bytes: Bytes = frame.into();
let frame: Frame = Frame::from(bytes);
```

### 2. Service Example (`service_example.rs`)

Shows how to build a Tower-compatible service that processes Frames:

```bash
cargo run --example service_example
```

**What you'll learn:**
- Implementing the Tower `Service` trait for Frame processing
- Adding middleware (buffering, concurrency limits, timeouts)
- Handling different Frame types (Exec, Msg, Ping/Pong, Json)
- Error handling in services
- Concurrent request processing
- Service cloning for parallel operations

**Key concepts:**
```rust
// Implement a service
impl Service<Frame> for MyService {
    type Response = Frame;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<Frame, BoxError>> + Send + Sync>>;

    fn call(&mut self, req: Frame) -> Self::Future {
        Box::pin(async move {
            match req {
                Frame::Exec(proc) => /* handle command */,
                Frame::Ping => Ok(Frame::Pong),
                _ => Ok(Frame::Error("Unsupported".to_string()))
            }
        })
    }
}

// Add middleware
let service = ServiceBuilder::new()
    .buffer(100)
    .concurrency_limit(10)
    .timeout(Duration::from_secs(5))
    .service(MyService);
```

### 3. Original Mako Example (`mako.rs`)

Real-world example using NATS for distributed communication:

```bash
# Requires NATS server running
cargo run --example mako
```

**What you'll learn:**
- Integration with NATS messaging
- Building a distributed time service
- Using Kong/Monkey pattern for pub/sub
- Combining reactor with external services

## Frame Protocol Reference

### Frame Variants

```rust
pub enum Frame {
    Ping,                          // Health check request
    Pong,                          // Health check response
    Msg(String),                   // Text message
    Bytes(Box<[u8]>),             // Binary data
    Json(JsonValue),               // JSON data
    Exec(Proc),                    // Command execution
    SendBox(SendBox),              // Message routing
    Close,                         // Close connection
    Fin,                           // Finished/complete
    Error(String),                 // Error message
}
```

### Common Operations

```rust
// Creating frames
let msg = Frame::message("hello");
let json = Frame::json(json!({"key": "value"}));
let cmd = Frame::exec("process", vec!["arg1", "arg2"]);
let ping = Frame::ping();

// Encoding/decoding
let bytes = frame.encode()?;              // Vec<u8>
let decoded = Frame::decode(&bytes)?;     // Frame

// Trait conversions
let bytes: Bytes = frame.into();          // Frame -> Bytes
let frame: Frame = Frame::from(bytes);    // Bytes -> Frame

// JSON extraction
if let Some(json) = frame.as_json() {
    // Use serde_json::Value
}
```

### Proc (Command Execution)

```rust
pub struct Proc {
    pub cmd: String,        // Command name
    pub args: Vec<String>,  // Arguments
}

// Usage
let proc = Proc::new("calculate", vec!["add", "10", "20"]);
let frame = Frame::Exec(proc);
```

### SendBox (Message Routing)

```rust
pub struct SendBox {
    pub from: String,       // Sender ID
    pub addr: String,       // Recipient ID
    pub data: Box<[u8]>,   // Payload
}

let sendbox = SendBox {
    from: "agent-001".to_string(),
    addr: "agent-002".to_string(),
    data: vec![1, 2, 3].into_boxed_slice(),
};
let frame = Frame::SendBox(sendbox);
```

## Tower Integration

### Basic Service Pattern

```rust
#[derive(Clone)]
struct MyService;

impl Service<Frame> for MyService {
    type Response = Frame;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<Frame, BoxError>> + Send + Sync>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Frame) -> Self::Future {
        Box::pin(async move {
            // Process request
            Ok(Frame::Pong)
        })
    }
}
```

### Middleware Stack

```rust
use tower::{ServiceBuilder, ServiceExt};

let service = ServiceBuilder::new()
    .buffer(100)                              // Request buffering
    .concurrency_limit(10)                    // Max concurrent requests
    .timeout(Duration::from_secs(5))          // Request timeout
    .rate_limit(100, Duration::from_secs(1))  // Rate limiting
    .service(MyService);
```

### Using the Service

```rust
// Single request
let mut service = service;
let response = service.ready().await?.call(Frame::Ping).await?;

// Concurrent requests
let mut svc = service.clone();
tokio::spawn(async move {
    svc.ready().await?.call(frame).await
});
```

## Performance Tips

1. **Use `From<Frame> for Bytes`**: More efficient than manual encoding
2. **Clone services**: Tower services are cheap to clone
3. **Buffer requests**: Use `.buffer()` middleware for bursty loads
4. **Set concurrency limits**: Prevent resource exhaustion
5. **Add timeouts**: Prevent hanging requests

## Common Patterns

### Request/Response

```rust
let request = Frame::exec("process", vec!["data"]);
let response = service.ready().await?.call(request).await?;
match response {
    Frame::Msg(result) => println!("Success: {}", result),
    Frame::Error(err) => eprintln!("Error: {}", err),
    _ => {}
}
```

### Health Checking

```rust
let health = service.ready().await?.call(Frame::Ping).await?;
assert_eq!(health, Frame::Pong);
```

### JSON RPC Style

```rust
let request = Frame::json(json!({
    "method": "calculate",
    "params": {"a": 10, "b": 20, "op": "add"}
}));
let response = service.ready().await?.call(request).await?;
if let Some(json) = response.as_json() {
    println!("Result: {}", json);
}
```

## Next Steps

- Check out the [main README](../README.md) for architecture details
- See `simian-nats-streams` for distributed messaging integration
- Look at `simian-base-api` for agent communication patterns
- Review tests in `src/protocol/frame.rs` for more examples

## Contributing

When adding new examples:
1. Keep them focused on one concept
2. Add comprehensive comments
3. Include error handling
4. Update this README
5. Test with `cargo run --example <name>`
