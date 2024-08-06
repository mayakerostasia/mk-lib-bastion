# bb-lib-bastion  
[![Docker](https://github.com/BlueBastion/DEV-bb-lib-bastion/actions/workflows/docker_build.yaml/badge.svg)](https://github.com/BlueBastion/DEV-bb-lib-bastion/actions/workflows/docker_build.yaml)
[![Rust Build](https://github.com/BlueBastion/DEV-bb-lib-bastion/actions/workflows/rust_build.yaml/badge.svg)](https://github.com/BlueBastion/DEV-bb-lib-bastion/actions/workflows/rust_build.yaml)  

This project is a collection of Rust libraries and binaries. The main components of the project are:

- [bb-bin-monkey](#bb-bin-monkey)
- [bb-lib-nats-streams](#bb-lib-nats-streams)
- [cfg_creator](bb-bin-cfg-creator)
- [bb-lib-surreal-client](#bb-lib-surreal-client)
- [bb-lib-config](#bb-lib-config)
- [bb-lib-reactor](#bb-lib-reactor)
- [bb-lib-event](#bb-lib-event)

## bb-bin-monkey
[Readme](./bb-bin-monkey/README.md) 
[Cargo](./bb-bin-monkey/Cargo.toml)

## bb-lib-base-api

## bb-lib-config

## bb-lib-event [unimplimented]
[Cargo](./bb-lib-event/Cargo.toml)

## bb-lib-http-listener
provides http endpoints healthz and readyz
```rust
use bb_lib_http_listener::Server;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let listener = Server::new("127.0.0.1:6060");
    let _: ! = listener.listen().await()?;
    Ok(())
}

```

## bb-lib-nats-streams
Tools for sending and receiving requests across a nats messaging service  
[Cargo](./bb-lib-nats-streams/Cargo.toml)

## bb-lib-reactor [this is an example repo]
Tower service creator  
[Cargo](./bb-lib-reactor/Cargo.toml)

## bb-lib-surreal-client
Surreal Database Client and Storage tools  
[Cargo](./bb-lib-surreal-client/Cargo.toml)

## bb-lib-tracing
Tracing instrumentation
[Cargo](./bb-lib-tracing/Cargo.toml)

### Usage:
Initialize the tracing subscribers using the following function in a tokio runtime  
```rust
// A tokio reactor must be enabled to initialize the tracing library
#[tokio::main]
async fn main() {
    let _otel_guard = bb_lib_tracing::initialize()?;
    // ... etc ... 
}
```
bump
