// Integration test for the event capture system
use simian_event::{
    capture::EventCapture, event_new::*, filter::SeverityFilter, sink::MemoryEventSink,
};
use std::sync::Arc;

#[tokio::test]
async fn test_full_event_capture_workflow() {
    let sink = Arc::new(MemoryEventSink::new());
    let events = EventCapture::new("integration-test").with_sink(sink.clone());

    // Test convenience methods
    events.agent_started("agent-1").await.unwrap();
    events
        .message_sent("alice", Some("bob"), "greeting", 1024)
        .await
        .unwrap();
    events
        .transport_connected("nats")
        .await
        .unwrap();

    // Verify events were captured
    let captured = sink.get_events().await;
    assert_eq!(captured.len(), 3);

    // Verify event details
    assert_eq!(captured[0].source, "integration-test");
    assert_eq!(captured[0].category, EventCategory::Agent);

    match &captured[1].data {
        EventData::MessageSent { from, to, size, .. } => {
            assert_eq!(from, "alice");
            assert_eq!(to.as_ref().map(|s| s.as_str()), Some("bob"));
            assert_eq!(*size, 1024);
        }
        _ => panic!("Expected MessageSent event"),
    }
}

#[tokio::test]
async fn test_event_filtering() {
    let sink = Arc::new(MemoryEventSink::new());
    let events = EventCapture::new("filtered-test")
        .with_sink(sink.clone())
        .with_filter(Arc::new(SeverityFilter::new(EventSeverity::Warn)));

    // Info events should be filtered out
    events.agent_started("agent-1").await.unwrap();
    events
        .message_sent("alice", Some("bob"), "msg", 100)
        .await
        .unwrap();

    // Error events should pass
    events
        .transport_error("connection failed")
        .await
        .unwrap();

    let captured = sink.get_events().await;
    assert_eq!(captured.len(), 1);
    assert_eq!(captured[0].severity, EventSeverity::Error);
}

#[tokio::test]
async fn test_event_serialization() {
    let event = Event::new(
        "test-source",
        EventCategory::Message,
        EventSeverity::Info,
        EventData::Text("test message".into()),
    );

    // Test JSON serialization
    let json = event.to_json().unwrap();
    let decoded = Event::from_json(&json).unwrap();
    assert_eq!(event.id, decoded.id);
    assert_eq!(event.source, decoded.source);

    // Test bincode serialization
    let bytes = event.to_bincode().unwrap();
    let decoded = Event::from_bincode(&bytes).unwrap();
    assert_eq!(event.id, decoded.id);
    assert_eq!(event.source, decoded.source);
}
