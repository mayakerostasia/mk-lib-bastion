# bb-lib-bastion

This project is a collection of Rust libraries and binaries. The main components of the project are:

- [bb-lib-nats-streams](bb-lib-nats-streams/Cargo.toml)
- [bb-lib-surreal-client](bb-lib-surreal-client/Cargo.toml)
- [bb-lib-config](bb-lib-config/Cargo.toml)
- [bb-lib-reactor](bb-lib-reactor/Cargo.toml)
- [bb-lib-event](bb-lib-event/Cargo.toml)
- [cfg_creator](bb-bin-cfg-creator/Cargo.toml)

## bb-lib-base-api

## bb-lib-config

## bb-lib-event

## bb-lib-http-listener

## bb-lib-nats-streams
Tools for sending and receiving requests across a nats messaging service

## bb-lib-reactor
Tower service creator

## bb-lib-surreal-client
Surreal Database Client and Storage tools


## bb-lib-tracing
Tracing instrumentation

```rust
// A tokio reactor must be enabled to initialize the tracing library
#[tokio::main]
async fn main() {
    let _otel_guard = bb_lib_tracing::initialize()?;
    // ... etc ... 
}
```
