// Example demonstrating the new v2 event capture system
use anyhow::Result;
use simian_event::{
    Event, EventCapture, EventCategory, EventData, EventSeverity,
    FileEventSink, MemoryEventSink, SeverityFilter,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Simian Event System v2 Demo ===\n");

    // Example 1: Memory sink for testing
    println!("1. Using Memory Sink:");
    demo_memory_sink().await?;

    // Example 2: File sink for production
    println!("\n2. Using File Sink:");
    demo_file_sink().await?;

    // Example 3: Filtering events
    println!("\n3. Filtering Events:");
    demo_filtering().await?;

    // Example 4: Multiple sinks
    println!("\n4. Multiple Sinks:");
    demo_multiple_sinks().await?;

    // Example 5: Custom events
    println!("\n5. Custom Events:");
    demo_custom_events().await?;

    println!("\n=== Demo Complete ===");
    Ok(())
}

async fn demo_memory_sink() -> Result<()> {
    let sink = Arc::new(MemoryEventSink::new());
    let events = EventCapture::new("demo-agent").with_sink(sink.clone());

    // Emit some events
    events.agent_started("demo-agent").await?;
    events.message_sent("alice", Some("bob"), "hello", 512).await?;
    events.transport_connected("nats").await?;

    // Get captured events
    let captured = sink.get_events().await;
    println!("  Captured {} events", captured.len());
    for (i, event) in captured.iter().enumerate() {
        println!("  Event {}: {:?} - {:?}", i + 1, event.category, event.severity);
    }

    Ok(())
}

async fn demo_file_sink() -> Result<()> {
    let temp_file = std::env::temp_dir().join("simian_events_demo.jsonl");
    let file_path = temp_file.to_str().unwrap();

    let sink = Arc::new(FileEventSink::new(file_path).await?);
    let events = EventCapture::new("file-demo").with_sink(sink);

    events.agent_started("file-demo").await?;
    events.message_sent("system", None, "broadcast", 256).await?;

    println!("  Events written to: {}", file_path);

    // Read back the file
    let content = tokio::fs::read_to_string(file_path).await?;
    println!("  File contains {} lines", content.lines().count());

    // Parse first event
    if let Some(first_line) = content.lines().next() {
        let event: Event = Event::from_json(first_line)?;
        println!("  First event: id={}, source={}", event.id, event.source);
    }

    // Cleanup
    let _ = tokio::fs::remove_file(file_path).await;

    Ok(())
}

async fn demo_filtering() -> Result<()> {
    let sink = Arc::new(MemoryEventSink::new());

    // Only capture warnings and errors
    let events = EventCapture::new("filtered-agent")
        .with_sink(sink.clone())
        .with_filter(Arc::new(SeverityFilter::new(EventSeverity::Warn)));

    // Emit events with different severities
    events.capture(
        EventCategory::System,
        EventSeverity::Info,
        EventData::Text("info message".into()),
    ).await?;

    events.capture(
        EventCategory::System,
        EventSeverity::Warn,
        EventData::Text("warning message".into()),
    ).await?;

    events.capture(
        EventCategory::System,
        EventSeverity::Error,
        EventData::Text("error message".into()),
    ).await?;

    let captured = sink.get_events().await;
    println!("  Emitted 3 events (Info, Warn, Error)");
    println!("  Filter captured {} events (Warn, Error only)", captured.len());
    assert_eq!(captured.len(), 2);

    Ok(())
}

async fn demo_multiple_sinks() -> Result<()> {
    let mem_sink = Arc::new(MemoryEventSink::new());

    let temp_file = std::env::temp_dir().join("simian_events_multi.jsonl");
    let file_sink = Arc::new(FileEventSink::new(temp_file.to_str().unwrap()).await?);

    // Send to both sinks
    let events = EventCapture::new("multi-sink")
        .with_sink(mem_sink.clone())
        .with_sink(file_sink);

    events.agent_started("multi-sink").await?;
    events.transport_connected("nats").await?;

    println!("  Events sent to both memory and file sinks");
    println!("  Memory sink has {} events", mem_sink.len().await);

    // Cleanup
    let _ = tokio::fs::remove_file(&temp_file).await;

    Ok(())
}

async fn demo_custom_events() -> Result<()> {
    let sink = Arc::new(MemoryEventSink::new());
    let events = EventCapture::new("custom-demo").with_sink(sink.clone());

    // Custom structured data
    events.capture(
        EventCategory::Custom("ml-inference".to_string()),
        EventSeverity::Info,
        EventData::Structured(serde_json::json!({
            "model": "gpt-4",
            "tokens": 1234,
            "latency_ms": 456,
            "cost_usd": 0.05
        })),
    ).await?;

    // Simple text event
    events.capture(
        EventCategory::System,
        EventSeverity::Debug,
        EventData::Text("Custom text event".to_string()),
    ).await?;

    let captured = sink.get_events().await;
    println!("  Captured {} custom events", captured.len());

    if let EventData::Structured(json) = &captured[0].data {
        println!("  Structured event data: {}", json);
    }

    Ok(())
}
