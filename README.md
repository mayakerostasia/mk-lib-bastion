# simian-bastion  

This project is a collection of Rust libraries and binaries for building robust agent-based systems.

## Main Components

- [simian-bin-monkey](#simian-bin-monkey) - A multi-transport agent communication tool.
- [simian-base-api](#simian-base-api) - Core Agent Communication Protocol (ACP) and high-level `SimianAgent`.
- [simian-nats-streams](#simian-nats-streams) - NATS-based transport for ACP.
- [simian-iggy-streams](#simian-iggy-streams) - Iggy-based transport for ACP.
- [simian-bin-cfg-creator](simian-bin-cfg-creator) - Configuration generation tool.
- [simian-surreal-client](#simian-surreal-client) - SurrealDB integration.
- [simian-tracing](#simian-tracing) - Tracing and observability.

## simian-base-api (ACP)
The core library defining the **Agent Communication Protocol (ACP)**. It provides:
- `Transport` trait for pluggable backends (NATS, Iggy, etc.).
- `AcpMessage` structure with support for Performatives (Request, Inform, Agree, etc.).
- `SimianAgent` high-level wrapper for easy agent development.

```rust
let transport = NatsTransport::new("nats://localhost:4222", "agents").await?;
let agent = SimianAgent::new(AgentId::new("my-agent"), transport);

// Send a request
agent.request(AgentId::new("target"), "ping", Bytes::from("hello")).await?;

// Listen for messages
let mut stream = agent.listen().await?;
while let Some(msg) = stream.next().await {
    println!("Received: {:?}