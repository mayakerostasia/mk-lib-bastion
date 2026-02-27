use crate::event_new::Event;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

/// Trait for event storage backends
#[async_trait]
pub trait EventSink: Send + Sync {
    /// Write an event to storage
    async fn emit(&self, event: Event) -> Result<()>;

    /// Flush any buffered events
    async fn flush(&self) -> Result<()>;

    /// Close the sink
    async fn close(&self) -> Result<()>;
}

/// Memory-based event sink for testing
#[derive(Debug, Clone)]
pub struct MemoryEventSink {
    events: Arc<Mutex<Vec<Event>>>,
}

impl MemoryEventSink {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Get all captured events
    pub async fn get_events(&self) -> Vec<Event> {
        self.events.lock().await.clone()
    }

    /// Clear all captured events
    pub async fn clear(&self) {
        self.events.lock().await.clear();
    }

    /// Get event count
    pub async fn len(&self) -> usize {
        self.events.lock().await.len()
    }

    /// Check if empty
    pub async fn is_empty(&self) -> bool {
        self.events.lock().await.is_empty()
    }
}

impl Default for MemoryEventSink {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventSink for MemoryEventSink {
    async fn emit(&self, event: Event) -> Result<()> {
        self.events.lock().await.push(event);
        Ok(())
    }

    async fn flush(&self) -> Result<()> {
        Ok(())
    }

    async fn close(&self) -> Result<()> {
        Ok(())
    }
}

/// File-based event sink (JSONL format)
pub struct FileEventSink {
    file: Arc<Mutex<File>>,
    path: String,
}

impl FileEventSink {
    /// Create a new file sink
    pub async fn new(path: impl Into<String>) -> Result<Self> {
        let path = path.into();
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .await?;

        Ok(Self {
            file: Arc::new(Mutex::new(file)),
            path,
        })
    }

    /// Get the file path
    pub fn path(&self) -> &str {
        &self.path
    }
}

#[async_trait]
impl EventSink for FileEventSink {
    async fn emit(&self, event: Event) -> Result<()> {
        let mut file = self.file.lock().await;
        let json = event.to_json()?;
        file.write_all(json.as_bytes()).await?;
        file.write_all(b"\n").await?;
        Ok(())
    }

    async fn flush(&self) -> Result<()> {
        self.file.lock().await.flush().await?;
        Ok(())
    }

    async fn close(&self) -> Result<()> {
        self.flush().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_new::{EventCategory, EventData, EventSeverity};

    #[tokio::test]
    async fn test_memory_sink() {
        let sink = MemoryEventSink::new();

        let event1 = Event::new(
            "test",
            EventCategory::System,
            EventSeverity::Info,
            EventData::Text("test1".into()),
        );

        let event2 = Event::new(
            "test",
            EventCategory::System,
            EventSeverity::Warn,
            EventData::Text("test2".into()),
        );

        sink.emit(event1.clone()).await.unwrap();
        sink.emit(event2.clone()).await.unwrap();

        let events = sink.get_events().await;
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].id, event1.id);
        assert_eq!(events[1].id, event2.id);

        assert!(!sink.is_empty().await);
        assert_eq!(sink.len().await, 2);

        sink.clear().await;
        assert!(sink.is_empty().await);
    }

    #[tokio::test]
    async fn test_memory_sink_flush_and_close() {
        let sink = MemoryEventSink::new();
        sink.flush().await.unwrap();
        sink.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_file_sink() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!("test_events_{}.jsonl", uuid::Uuid::new_v4()));
        let file_path_str = file_path.to_str().unwrap();

        let sink = FileEventSink::new(file_path_str).await.unwrap();

        let event = Event::new(
            "test",
            EventCategory::Agent,
            EventSeverity::Info,
            EventData::AgentStarted {
                agent_id: "agent-1".to_string(),
            },
        );

        sink.emit(event.clone()).await.unwrap();
        sink.flush().await.unwrap();

        // Read the file and verify content
        let content = tokio::fs::read_to_string(file_path_str).await.unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 1);

        let decoded = Event::from_json(lines[0]).unwrap();
        assert_eq!(decoded.id, event.id);
        assert_eq!(decoded.source, event.source);

        sink.close().await.unwrap();

        // Cleanup
        let _ = tokio::fs::remove_file(file_path_str).await;
    }

    #[tokio::test]
    async fn test_file_sink_multiple_events() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!("test_events_{}.jsonl", uuid::Uuid::new_v4()));
        let file_path_str = file_path.to_str().unwrap();

        let sink = FileEventSink::new(file_path_str).await.unwrap();

        for i in 0..5 {
            let event = Event::new(
                "test",
                EventCategory::System,
                EventSeverity::Info,
                EventData::Text(format!("event-{}", i)),
            );
            sink.emit(event).await.unwrap();
        }

        sink.flush().await.unwrap();

        let content = tokio::fs::read_to_string(file_path_str).await.unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 5);

        sink.close().await.unwrap();

        // Cleanup
        let _ = tokio::fs::remove_file(file_path_str).await;
    }
}
