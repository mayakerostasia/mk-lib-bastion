# Event System Integration Guide

## Overview

The enhanced event system (EventCapture, EventSink, EventFilter) is now available in simian-event but is **NOT** automatically integrated into transport layers to avoid circular dependencies.

## Integration Pattern

To use event capture with transports, follow this pattern:

### 1. Create Event Capture in Your Application

```rust
use simian_event::{EventCapture, MemoryEventSink, FileEventSink, SeverityFilter, EventSeverity};
use std::sync::Arc;

// Create sinks
let memory_sink = Arc::new(MemoryEventSink::new(1000)); // Keep last 1000 events
let file_sink = Arc::new(FileEventSink::new("events.jsonl").await?);

// Create capture with filter
let event_capture = Arc::new(
    EventCapture::new("my-service")
        .with_sink(memory_sink.clone())
        .with_sink(file_sink)
        .with_filter(Arc::new(SeverityFilter::new(EventSeverity::Info)))
);
```

### 2. Manually Capture Transport Events

```rust
use simian_base_api::transport::Transport;
use simian_nats_streams::NatsTransport;
use simian_event::{EventCategory, EventSeverity, EventData};

let transport = NatsTransport::new("nats://localhost:4222", "agents").await?;

// Before publishing
event_capture.capture(
    EventCategory::Transport,
    EventSeverity::Debug,
    EventData::MessageSent {
        from: "agent-1".to_string(),
        to: "agent-2".to_string(),
        message_id: msg.message_id.to_string(),
    },
).await?;

transport.publish(message).await?;
```

### 3. Alternative: Wrapper Pattern

Create a wrapper that adds event capture:

```rust
pub struct ObservableTransport<T: Transport> {
    inner: T,
    event_capture: Arc<EventCapture>,
}

#[async_trait]
impl<T: Transport> Transport for ObservableTransport<T> {
    async fn publish(&self, message: AcpMessage) -> Result<()> {
        // Capture event
        self.event_capture.capture(
            EventCategory::Transport,
            EventSeverity::Debug,
            EventData::MessageSent { /* ... */ },
        ).await?;
        
        // Delegate to inner transport
        self.inner.publish(message).await
    }
    
    // ... other methods
}
```

## Why Not Built-In?

- **Circular dependency**: simian-event uses simian-nats-streams for its own examples/tests
- **Flexibility**: Not all users need observability overhead
- **Separation of concerns**: Transport layer remains lightweight and focused

## Future Work

Consider:
1. Moving example/test dependencies out of simian-event to break the cycle
2. Feature flags to conditionally enable event hooks
3. A separate "simian-observability" crate that bridges event + transport layers
