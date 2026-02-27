use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Enhanced event structure with metadata and categorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub timestamp: i64,
    pub source: String,
    pub category: EventCategory,
    pub severity: EventSeverity,
    pub data: EventData,
    pub metadata: HashMap<String, String>,
}

impl Event {
    /// Create a new event
    pub fn new(
        source: impl Into<String>,
        category: EventCategory,
        severity: EventSeverity,
        data: EventData,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: chrono::Utc::now().timestamp(),
            source: source.into(),
            category,
            severity,
            data,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the event
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Encode event to JSON
    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    /// Decode event from JSON
    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(json)?)
    }

    /// Encode event to bincode
    pub fn to_bincode(&self) -> anyhow::Result<Vec<u8>> {
        Ok(bincode::serialize(self)?)
    }

    /// Decode event from bincode
    pub fn from_bincode(data: &[u8]) -> anyhow::Result<Self> {
        Ok(bincode::deserialize(data)?)
    }
}

/// Event category classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EventCategory {
    Message,
    Transport,
    Agent,
    Service,
    System,
    Custom(String),
}

/// Event severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventSeverity {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Event data payload variants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventData {
    // Message events
    MessageSent {
        from: String,
        to: Option<String>,
        subject: String,
        size: usize,
    },
    MessageReceived {
        from: String,
        to: String,
        subject: String,
        size: usize,
    },

    // Transport events
    TransportConnected {
        backend: String,
    },
    TransportDisconnected {
        backend: String,
        reason: String,
    },
    TransportError {
        error: String,
    },

    // Agent events
    AgentStarted {
        agent_id: String,
    },
    AgentStopped {
        agent_id: String,
    },
    AgentError {
        agent_id: String,
        error: String,
    },

    // Service events
    ServiceRequestReceived {
        service: String,
        method: String,
    },
    ServiceRequestCompleted {
        service: String,
        method: String,
        duration_ms: u64,
    },
    ServiceRequestFailed {
        service: String,
        method: String,
        error: String,
    },

    // Generic
    Text(String),
    Structured(serde_json::Value),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let event = Event::new(
            "test-source",
            EventCategory::Message,
            EventSeverity::Info,
            EventData::Text("test".into()),
        );

        assert_eq!(event.source, "test-source");
        assert_eq!(event.category, EventCategory::Message);
        assert_eq!(event.severity, EventSeverity::Info);
    }

    #[test]
    fn test_event_with_metadata() {
        let event = Event::new(
            "test",
            EventCategory::System,
            EventSeverity::Debug,
            EventData::Text("test".into()),
        )
        .with_metadata("key1", "value1")
        .with_metadata("key2", "value2");

        assert_eq!(event.metadata.len(), 2);
        assert_eq!(event.metadata.get("key1"), Some(&"value1".to_string()));
    }

    #[test]
    fn test_event_json_encoding() {
        let event = Event::new(
            "test",
            EventCategory::Agent,
            EventSeverity::Info,
            EventData::AgentStarted {
                agent_id: "agent-1".to_string(),
            },
        );

        let json = event.to_json().unwrap();
        let decoded = Event::from_json(&json).unwrap();

        assert_eq!(event.id, decoded.id);
        assert_eq!(event.source, decoded.source);
        assert_eq!(event.category, decoded.category);
    }

    #[test]
    fn test_event_bincode_encoding() {
        let event = Event::new(
            "test",
            EventCategory::Transport,
            EventSeverity::Error,
            EventData::TransportError {
                error: "connection lost".to_string(),
            },
        );

        let encoded = event.to_bincode().unwrap();
        let decoded = Event::from_bincode(&encoded).unwrap();

        assert_eq!(event.id, decoded.id);
        assert_eq!(event.source, decoded.source);
    }

    #[test]
    fn test_severity_ordering() {
        assert!(EventSeverity::Trace < EventSeverity::Debug);
        assert!(EventSeverity::Debug < EventSeverity::Info);
        assert!(EventSeverity::Info < EventSeverity::Warn);
        assert!(EventSeverity::Warn < EventSeverity::Error);
    }

    #[test]
    fn test_category_equality() {
        assert_eq!(EventCategory::Message, EventCategory::Message);
        assert_ne!(EventCategory::Message, EventCategory::Agent);
        assert_eq!(
            EventCategory::Custom("test".into()),
            EventCategory::Custom("test".into())
        );
    }

    #[test]
    fn test_message_sent_event() {
        let event = Event::new(
            "agent-1",
            EventCategory::Message,
            EventSeverity::Info,
            EventData::MessageSent {
                from: "alice".to_string(),
                to: Some("bob".to_string()),
                subject: "greeting".to_string(),
                size: 1024,
            },
        );

        if let EventData::MessageSent { from, to, size, .. } = event.data {
            assert_eq!(from, "alice");
            assert_eq!(to, Some("bob".to_string()));
            assert_eq!(size, 1024);
        } else {
            panic!("Expected MessageSent event");
        }
    }
}
