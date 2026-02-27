use crate::event_new::Event;
use crate::filter::EventFilter;
use crate::sink::EventSink;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// The main entry point for event observability
#[derive(Clone)]
pub struct EventCapture {
    source: String,
    sinks: Vec<Arc<dyn EventSink>>,
    filter: Arc<dyn EventFilter>,
}

impl EventCapture {
    /// Create a new EventCapture with the given source identifier
    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_string(),
            sinks: vec![],
            filter: Arc::new(crate::filter::AllowAllFilter),
        }
    }

    /// Add a sink to emit events to
    pub fn with_sink(mut self, sink: Arc<dyn EventSink>) -> Self {
        self.sinks.push(sink);
        self
    }

    /// Set a filter for events
    pub fn with_filter(mut self, filter: Arc<dyn EventFilter>) -> Self {
        self.filter = filter;
        self
    }

    /// Capture an event with the given category, severity, and data
    pub async fn capture(
        &self,
        category: crate::event_new::EventCategory,
        severity: crate::event_new::EventSeverity,
        data: crate::event_new::EventData,
    ) -> Result<()> {
        let event = Event {
            id: Uuid::new_v4(),
            timestamp: chrono::Utc::now().timestamp(),
            source: self.source.clone(),
            category,
            severity,
            data,
            metadata: HashMap::new(),
        };

        if !self.filter.should_capture(&event) {
            return Ok(());
        }

        for sink in &self.sinks {
            sink.emit(event.clone()).await?;
        }

        Ok(())
    }

    /// Convenience: capture a message sent event
    pub async fn message_sent(
        &self,
        from: &str,
        to: Option<&str>,
        subject: &str,
        size: usize,
    ) -> Result<()> {
        self.capture(
            crate::event_new::EventCategory::Message,
            crate::event_new::EventSeverity::Info,
            crate::event_new::EventData::MessageSent {
                from: from.to_string(),
                to: to.map(|s| s.to_string()),
                subject: subject.to_string(),
                size,
            },
        )
        .await
    }

    /// Convenience: capture a message received event
    pub async fn message_received(
        &self,
        from: &str,
        to: &str,
        subject: &str,
        size: usize,
    ) -> Result<()> {
        self.capture(
            crate::event_new::EventCategory::Message,
            crate::event_new::EventSeverity::Info,
            crate::event_new::EventData::MessageReceived {
                from: from.to_string(),
                to: to.to_string(),
                subject: subject.to_string(),
                size,
            },
        )
        .await
    }

    /// Convenience: capture an agent started event
    pub async fn agent_started(&self, agent_id: &str) -> Result<()> {
        self.capture(
            crate::event_new::EventCategory::Agent,
            crate::event_new::EventSeverity::Info,
            crate::event_new::EventData::AgentStarted {
                agent_id: agent_id.to_string(),
            },
        )
        .await
    }

    /// Convenience: capture an agent stopped event
    pub async fn agent_stopped(&self, agent_id: &str) -> Result<()> {
        self.capture(
            crate::event_new::EventCategory::Agent,
            crate::event_new::EventSeverity::Info,
            crate::event_new::EventData::AgentStopped {
                agent_id: agent_id.to_string(),
            },
        )
        .await
    }

    /// Convenience: capture a transport connected event
    pub async fn transport_connected(&self, backend: &str) -> Result<()> {
        self.capture(
            crate::event_new::EventCategory::Transport,
            crate::event_new::EventSeverity::Info,
            crate::event_new::EventData::TransportConnected {
                backend: backend.to_string(),
            },
        )
        .await
    }

    /// Convenience: capture a transport error event
    pub async fn transport_error(&self, error: &str) -> Result<()> {
        self.capture(
            crate::event_new::EventCategory::Transport,
            crate::event_new::EventSeverity::Error,
            crate::event_new::EventData::TransportError {
                error: error.to_string(),
            },
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_new::{EventCategory, EventData, EventSeverity};
    use crate::filter::SeverityFilter;
    use crate::sink::MemoryEventSink;

    #[tokio::test]
    async fn test_event_capture_with_memory_sink() {
        let sink = Arc::new(MemoryEventSink::new());
        let events = EventCapture::new("test").with_sink(sink.clone());

        events
            .message_sent("alice", Some("bob"), "test", 100)
            .await
            .unwrap();

        let captured = sink.get_events().await;
        assert_eq!(captured.len(), 1);
        assert_eq!(captured[0].source, "test");

        if let EventData::MessageSent { from, to, .. } = &captured[0].data {
            assert_eq!(from, "alice");
            assert_eq!(to.as_ref().map(|s| s.as_str()), Some("bob"));
        } else {
            panic!("Expected MessageSent event");
        }
    }

    #[tokio::test]
    async fn test_severity_filter() {
        let sink = Arc::new(MemoryEventSink::new());
        let events = EventCapture::new("test")
            .with_sink(sink.clone())
            .with_filter(Arc::new(SeverityFilter::new(EventSeverity::Warn)));

        // Info should be filtered out
        events
            .capture(
                EventCategory::System,
                EventSeverity::Info,
                EventData::Text("info".into()),
            )
            .await
            .unwrap();

        // Warn should pass
        events
            .capture(
                EventCategory::System,
                EventSeverity::Warn,
                EventData::Text("warn".into()),
            )
            .await
            .unwrap();

        let captured = sink.get_events().await;
        assert_eq!(captured.len(), 1);
        assert_eq!(captured[0].severity, EventSeverity::Warn);
    }

    #[tokio::test]
    async fn test_multiple_sinks() {
        let sink1 = Arc::new(MemoryEventSink::new());
        let sink2 = Arc::new(MemoryEventSink::new());
        let events = EventCapture::new("test")
            .with_sink(sink1.clone())
            .with_sink(sink2.clone());

        events.agent_started("agent-1").await.unwrap();

        assert_eq!(sink1.get_events().await.len(), 1);
        assert_eq!(sink2.get_events().await.len(), 1);
    }

    #[tokio::test]
    async fn test_convenience_methods() {
        let sink = Arc::new(MemoryEventSink::new());
        let events = EventCapture::new("test").with_sink(sink.clone());

        events.agent_started("agent-1").await.unwrap();
        events.agent_stopped("agent-1").await.unwrap();
        events.transport_connected("nats").await.unwrap();
        events.transport_error("connection lost").await.unwrap();
        events
            .message_received("alice", "bob", "subject", 100)
            .await
            .unwrap();

        let captured = sink.get_events().await;
        assert_eq!(captured.len(), 5);
    }
}
