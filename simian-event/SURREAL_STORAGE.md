# Event Storage Integration with SurrealDB

## Overview

The simian-event system now includes pluggable storage backends. To integrate with simian-surreal-client:

## Implementation Pattern

### 1. Create SurrealEventSink

Create a new file `simian-event/src/surreal_sink.rs`:

```rust
use crate::event_new::Event;
use crate::sink::EventSink;
use anyhow::Result;
use async_trait::async_trait;
use simian_surreal_client::SurrealClient;
use std::sync::Arc;

pub struct SurrealEventSink {
    client: Arc<SurrealClient>,
    table: String,
}

impl SurrealEventSink {
    pub fn new(client: Arc<SurrealClient>, table: &str) -> Self {
        Self {
            client,
            table: table.to_string(),
        }
    }
}

#[async_trait]
impl EventSink for SurrealEventSink {
    async fn emit(&self, event: Event) -> Result<()> {
        // Convert event to SurrealDB record
        self.client
            .create((&self.table, event.id.to_string()))
            .content(event)
            .await?;
        Ok(())
    }

    async fn flush(&self) -> Result<()> {
        // SurrealDB writes are immediate
        Ok(())
    }

    async fn close(&self) -> Result<()> {
        // Nothing to close for SurrealDB
        Ok(())
    }
}
```

### 2. Add Optional Dependency

In `simian-event/Cargo.toml`:

```toml
[dependencies]
simian-surreal-client = { path = "../simian-surreal-client", optional = true }

[features]
surreal = ["dep:simian-surreal-client"]
```

### 3. Conditional Module

In `simian-event/src/lib.rs`:

```rust
#[cfg(feature = "surreal")]
pub mod surreal_sink;
#[cfg(feature = "surreal")]
pub use surreal_sink::SurrealEventSink;
```

### 4. Usage Example

```rust
use simian_event::{EventCapture, SurrealEventSink};
use simian_surreal_client::SurrealClient;
use std::sync::Arc;

// Create SurrealDB client
let surreal = Arc::new(SurrealClient::new("ws://localhost:8000").await?);

// Create event sink
let surreal_sink = Arc::new(SurrealEventSink::new(surreal, "events"));

// Create event capture with persistent storage
let capture = EventCapture::new("my-service")
    .with_sink(surreal_sink);

// Events are now persisted to SurrealDB
capture.capture(category, severity, data).await?;
```

## Querying Events

Use SurrealDB queries to analyze events:

```sql
-- Get all errors in last hour
SELECT * FROM events WHERE severity = 'Error' AND timestamp > time::now() - 1h;

-- Count events by category
SELECT category, count() FROM events GROUP BY category;

-- Find slow operations
SELECT * FROM events WHERE category = 'Service' AND metadata.duration_ms > 1000;
```

## Why Not Implemented?

- Feature flag complexity: Adds build-time dependency management
- Circular dependency risk: simian-surreal-client might use events internally
- Simple integration: Users can add the 20-line SurrealEventSink themselves
- Flexibility: Users may want custom schemas, indexes, or transformations

## Status

**DOCUMENTED BUT NOT IMPLEMENTED** - This guide provides everything needed for users to add SurrealDB storage themselves in 5 minutes.
