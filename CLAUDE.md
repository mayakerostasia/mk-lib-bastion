# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
# Build entire workspace
cargo build --workspace

# Build a specific crate
cargo build -p simian-reactor

# Check without building (fast)
cargo check --workspace

# Run all tests
cargo test --workspace --verbose

# Run tests for a specific crate
cargo test -p simian-nats-streams

# Run a single test by name
cargo test -p simian-reactor test_battery_capacity

# Format
cargo fmt

# Lint
cargo clippy --workspace
```

## Architecture

**simian-bastion** is a Rust workspace for building agent-based distributed systems. The design separates *what agents say* (ACP protocol) from *how they say it* (Transport trait), and composes middleware via Tower.

### Crate Dependency Flow

```
simian-base-api        ← defines Transport trait, AcpMessage, SimianAgent<T>
    ↑
simian-nats-streams    ← NatsTransport + Frame protocol + Kong/Monkey/KingKong
    ↑
simian-reactor         ← Tower middleware (MakoReactor, MakoLayer, MakoBattery)
                         + now owns the Frame/Proc protocol (simian-reactor::protocol)
```

Supporting crates (no strict ordering): `simian-event`, `simian-config`, `simian-tracing`, `simian-metrics`, `simian-surreal-client`, `simian-http-listener`, `simian-llm`, `simian-viz-bridge`.

Binaries: `simian-bin-agent` (long-lived agent server), `simian-bin-monkey` (CLI NATS sender), `simian-bin-cfg-creator`.

### Active Refactor: `feature/simian-mesh-refactor`

The Frame protocol is being **migrated from `simian-nats-streams` into `simian-reactor::protocol`**. Key changes in progress:

- `simian-reactor/src/protocol/frame.rs` now defines `Frame` and `Proc` using `rkyv` for serialization (replaces `bincode` that was in `simian-nats-streams`)
- `simian-nats-streams` will consume `simian-reactor::protocol::Frame` rather than defining its own
- **`simian-nats-streams` is the primary production transport — avoid breaking it**

### Two Messaging Layers

**ACP layer** (`simian-base-api`): High-level agent semantics.
- `Transport` trait: `publish(AcpMessage)`, `subscribe(agent_id) -> BoxStream<AcpMessage>`
- `SimianAgent<T>`: convenience wrapper with `request()`, `inform()`, `reply()`, `listen()`
- `AcpMessage`: UUID-keyed, performative-typed (Request, Inform, Query, Agree, Refuse, Failure)
- NATS subjects: `{base}.{agent_id}.inbox` (direct), `{base}.broadcast`

**Frame layer** (`simian-nats-streams` / migrating to `simian-reactor::protocol`): Structured NATS RPC.
- `Frame` enum: `Ping`, `Pong`, `Msg(String)`, `Bytes`, `Json`, `Exec(Proc)`, `SendBox`, `Fin`, `Error`, `Close`
- `Proc`: named RPC call `{ cmd: String, args: Vec<String> }`
- Frame → Bytes via rkyv (zero-copy deserialization)
- Kong/Monkey/KingKong: server/client/orchestrator for Frame-based NATS services

### Kong / Monkey / KingKong Pattern

- **Kong** — subscribes on a NATS subject, dispatches incoming `Frame`s to a handler or Tower service
- **Monkey** — publishes `Frame`s to a NATS subject, supports request-reply with optional timeout
- **KingKong** — manages multiple Kongs under a subject prefix, includes health check endpoint, and implements `Service<Frame>` (Tower-composable since recent work)

```rust
// KingKong as a Tower service (new capability)
let svc = ServiceBuilder::new()
    .buffer(10)
    .timeout(Duration::from_secs(30))
    .service(kk);  // kk: KingKong
```

### simian-reactor Tower Middleware

- **`MakoBattery`**: Semaphore-based token-bucket rate limiter. Permits replenish on a timer tick. Use inside agents for backpressure.
- **`ReactorCore`**: Named function registry (name → async fn). Dispatches `Frame::Exec(Proc)` to handlers.
- **`MakoReactor<S>`**: Tower `Service` wrapping an inner service + `ReactorCore`. Routes `Frame::Exec` to registry, delegates other frames to inner service.
- **`MakoLayer`**: Tower `Layer` that wraps any service with `MakoReactor`.
- **`FrameFuture`**: Pin-projected future wrapper for ergonomic `Service::Future` impls.

### Conventions

- `simian-*` = library crates; `simian-bin-*` = binary crates
- Crate names use underscores in Rust: `simian_base_api`, `simian_nats_streams`
- All workspace dependencies are in root `Cargo.toml` `[workspace.dependencies]`; crates use `workspace = true`
- Error handling: `anyhow::Result` for public APIs, `thiserror::Error` for domain errors, `tracing::error!` for logging
- Transport impls must be `Clone + Debug + Send + Sync`; return `BoxStream<'static, AcpMessage>` from subscribe
- Tower services use `Pin<Box<dyn Future>>` for `Service::Future`
- Unit tests live in `#[cfg(test)]` modules; integration tests in `tests/`; examples in `examples/`

### Local Development Infrastructure

NATS is required for most integration tests. Start it via Docker:

```bash
docker compose up nats -d          # just NATS
docker compose up                  # full multi-agent stack
```

The `simian-bin-monkey` CLI is the primary tool for manually testing Kong services:

```bash
cargo run -p simian-bin-monkey -- --subject agents.agent-001 --cmd uppercase --args "hello"
```

`simian-bin-agent` supports role-based behavior via `AGENT_ROLE` env var (`echo`, `processor`, `generator`, `verifier`) and exposes a health endpoint at `HEALTH_BIND` (default `0.0.0.0:6660`).

### Future Refactoring (Not Yet Implemented)

`FUTURE_REFACTORING.md` documents a planned v3.0.0 split of `simian-base-api` into:
- `simian-protocol` (protocol definitions only, minimal deps)
- `simian-base-api-client` (agent helper, depends on protocol)

This is deferred — do not attempt it without explicit instruction.
