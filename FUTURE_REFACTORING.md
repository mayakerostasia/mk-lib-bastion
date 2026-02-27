# Base API Refactoring - Future Work

## Current State

The `simian-base-api` crate currently mixes multiple concerns:
- **Protocol definitions**: `AcpMessage`, performatives, message types
- **Transport abstraction**: `Transport` trait
- **Agent API**: `SimianAgent` wrapper around transport

## Proposed Refactoring (Future v3.0.0)

### Phase 1: Create simian-protocol Crate

Extract protocol definitions into a new lightweight crate:

```
simian-protocol/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── agent.rs          # AgentId
    ├── transport.rs      # Transport trait (interface only)
    └── message.rs        # AcpMessage, Performative, etc.
```

**Benefits**:
- Minimal dependencies (just serde, uuid, chrono, bytes)
- Can be used without transport implementations
- Clear separation of "what" (protocol) vs "how" (transport)

### Phase 2: Update simian-base-api

After simian-protocol exists, slim down simian-base-api to just agent helpers:

```
simian-base-api/
├── Cargo.toml  (depends on simian-protocol)
└── src/
    ├── lib.rs
    └── agent.rs          # SimianAgent helper
```

### Phase 3: Rename to simian-base-api-client

Once slimmed down, rename to reflect its focused purpose:

```bash
mv simian-base-api simian-base-api-client
# Update all workspace references
# Update documentation
```

### Phase 4: Migration

Update all dependent crates:

```rust
// Old
use simian_base_api::transport::{Transport, AcpMessage};
use simian_base_api::agent::SimianAgent;

// New
use simian_protocol::{Transport, AcpMessage};  // Protocol definitions
use simian_base_api_client::SimianAgent;       // Agent helpers
```

## Why Not Now?

This refactoring was **attempted by an agent but failed due to rate limits**. The changes are extensive:

- 20+ files need import path updates
- Risk of breaking existing code
- Requires thorough testing
- Better done manually with careful review

## Blocked Tasks

- `baseapi-refactor` - Create simian-protocol, move types
- `baseapi-rename` - Rename to simian-base-api-client

## Status

**DOCUMENTED FOR FUTURE RELEASE** - This is good technical debt to address in v3.0.0, but not critical for v2.1.0.

The current organization works fine, it's just not as clean as it could be. This refactoring improves clarity but doesn't add features.

## Alternative: Leave As-Is

The current structure is functional and users are familiar with it. Consider leaving it unchanged unless there's a concrete problem to solve.

**Recommendation**: Skip this refactoring for v2.1.0. Revisit in v3.0.0 if there's user demand.
