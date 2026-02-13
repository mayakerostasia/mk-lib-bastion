# bb-lib-bastion  
[![Docker](https://github.com/BlueBastion/DEV-bb-lib-bastion/actions/workflows/docker_build.yaml/badge.svg)](https://github.com/BlueBastion/DEV-bb-lib-bastion/actions/workflows/docker_build.yaml)  
[![Rust Build](https://github.com/BlueBastion/DEV-bb-lib-bastion/actions/workflows/rust_build.yaml/badge.svg)](https://github.com/BlueBastion/DEV-bb-lib-bastion/actions/workflows/rust_build.yaml)  

This project is a collection of Rust libraries and binaries for building robust agent-based systems.

## Main Components

- [bb-bin-monkey](#bb-bin-monkey) - A multi-transport agent communication tool.
- [bb-lib-base-api](#bb-lib-base-api) - Core Agent Communication Protocol (ACP) and high-level `BastionAgent`.
- [bb-lib-nats-streams](#bb-lib-nats-streams) - NATS-based transport for ACP.
- [bb-lib-iggy-streams](#bb-lib-iggy-streams) - Iggy-based transport for ACP.
- [bb-bin-cfg-creator](bb-bin-cfg-creator) - Configuration generation tool.
- [bb-lib-surreal-client](#bb-lib-surreal-client) - SurrealDB integration.
- [bb-lib-tracing](#bb-lib-tracing) - Tracing and observability.

## bb-lib-base-api (ACP)
The core library defining the **Agent Communication Protocol (ACP)**. It provides:
- `Transport` trait for pluggable backends (NATS, Iggy, etc.).
- `AcpMessage` structure with support for Performatives (Request, Inform, Agree, etc.).
- `BastionAgent` high-level wrapper for easy agent development.

```rust
let transport = NatsTransport::new("nats://localhost:4222", "agents").await?;
let agent = BastionAgent::new(AgentId::new("my-agent"), transport);

// Send a request
agent.request(AgentId::new("target"), "ping", Bytes::from("hello")).await?;

// Listen for messages
let mut stream = agent.listen().await?;
while let Some(msg) = stream.next().await {
    println!("Received: {:?}", msg);
}
```

## bb-bin-monkey
A versatile CLI tool to interact with ACP agents over NATS or Iggy.
[Monkey Readme](./bb-bin-monkey/README.md)

## bb-lib-nats-streams
Provides `NatsTransport` for ACP.
[Cargo](./bb-lib-nats-streams/Cargo.toml)

## bb-lib-iggy-streams
Provides `IggyTransport` for ACP, leveraging Iggy's high-performance streaming capabilities.
[Cargo](./bb-lib-iggy-streams/Cargo.toml)

## bb-lib-tracing
Tracing instrumentation.
```rust
#[tokio::main]
async fn main() {
    let _otel_guard = bb_lib_tracing::initialize()?;
}
```