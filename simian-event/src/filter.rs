use crate::event_new::{Event, EventCategory, EventSeverity};
use std::collections::HashSet;
use std::sync::Arc;

/// Trait for filtering events
pub trait EventFilter: Send + Sync {
    fn should_capture(&self, event: &Event) -> bool;
}

/// Filter that allows all events
pub struct AllowAllFilter;

impl EventFilter for AllowAllFilter {
    fn should_capture(&self, _event: &Event) -> bool {
        true
    }
}

/// Filter by minimum severity level
pub struct SeverityFilter {
    min_severity: EventSeverity,
}

impl SeverityFilter {
    pub fn new(min_severity: EventSeverity) -> Self {
        Self { min_severity }
    }
}

impl EventFilter for SeverityFilter {
    fn should_capture(&self, event: &Event) -> bool {
        event.severity >= self.min_severity
    }
}

/// Filter by allowed categories
pub struct CategoryFilter {
    allowed: HashSet<EventCategory>,
}

impl CategoryFilter {
    pub fn new(allowed: Vec<EventCategory>) -> Self {
        Self {
            allowed: allowed.into_iter().collect(),
        }
    }

    pub fn allow(mut self, category: EventCategory) -> Self {
        self.allowed.insert(category);
        self
    }
}

impl EventFilter for CategoryFilter {
    fn should_capture(&self, event: &Event) -> bool {
        self.allowed.contains(&event.category)
    }
}

/// Filter by source prefix
pub struct SourceFilter {
    prefix: String,
}

impl SourceFilter {
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
        }
    }
}

impl EventFilter for SourceFilter {
    fn should_capture(&self, event: &Event) -> bool {
        event.source.starts_with(&self.prefix)
    }
}

/// Combine multiple filters with AND logic
pub struct AndFilter {
    filters: Vec<Arc<dyn EventFilter>>,
}

impl AndFilter {
    pub fn new(filters: Vec<Arc<dyn EventFilter>>) -> Self {
        Self { filters }
    }

    pub fn add(mut self, filter: Arc<dyn EventFilter>) -> Self {
        self.filters.push(filter);
        self
    }
}

impl EventFilter for AndFilter {
    fn should_capture(&self, event: &Event) -> bool {
        self.filters
            .iter()
            .all(|filter| filter.should_capture(event))
    }
}

/// Combine multiple filters with OR logic
pub struct OrFilter {
    filters: Vec<Arc<dyn EventFilter>>,
}

impl OrFilter {
    pub fn new(filters: Vec<Arc<dyn EventFilter>>) -> Self {
        Self { filters }
    }

    pub fn add(mut self, filter: Arc<dyn EventFilter>) -> Self {
        self.filters.push(filter);
        self
    }
}

impl EventFilter for OrFilter {
    fn should_capture(&self, event: &Event) -> bool {
        self.filters
            .iter()
            .any(|filter| filter.should_capture(event))
    }
}

/// Invert a filter (NOT logic)
pub struct NotFilter {
    inner: Arc<dyn EventFilter>,
}

impl NotFilter {
    pub fn new(inner: Arc<dyn EventFilter>) -> Self {
        Self { inner }
    }
}

impl EventFilter for NotFilter {
    fn should_capture(&self, event: &Event) -> bool {
        !self.inner.should_capture(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_new::EventData;

    #[test]
    fn test_allow_all_filter() {
        let filter = AllowAllFilter;
        let event = Event::new(
            "test",
            EventCategory::System,
            EventSeverity::Info,
            EventData::Text("test".into()),
        );

        assert!(filter.should_capture(&event));
    }

    #[test]
    fn test_severity_filter() {
        let filter = SeverityFilter::new(EventSeverity::Warn);

        let info_event = Event::new(
            "test",
            EventCategory::System,
            EventSeverity::Info,
            EventData::Text("info".into()),
        );

        let warn_event = Event::new(
            "test",
            EventCategory::System,
            EventSeverity::Warn,
            EventData::Text("warn".into()),
        );

        let error_event = Event::new(
            "test",
            EventCategory::System,
            EventSeverity::Error,
            EventData::Text("error".into()),
        );

        assert!(!filter.should_capture(&info_event));
        assert!(filter.should_capture(&warn_event));
        assert!(filter.should_capture(&error_event));
    }

    #[test]
    fn test_category_filter() {
        let filter =
            CategoryFilter::new(vec![EventCategory::Message, EventCategory::Agent]);

        let message_event = Event::new(
            "test",
            EventCategory::Message,
            EventSeverity::Info,
            EventData::Text("msg".into()),
        );

        let agent_event = Event::new(
            "test",
            EventCategory::Agent,
            EventSeverity::Info,
            EventData::Text("agent".into()),
        );

        let system_event = Event::new(
            "test",
            EventCategory::System,
            EventSeverity::Info,
            EventData::Text("system".into()),
        );

        assert!(filter.should_capture(&message_event));
        assert!(filter.should_capture(&agent_event));
        assert!(!filter.should_capture(&system_event));
    }

    #[test]
    fn test_source_filter() {
        let filter = SourceFilter::new("agent:");

        let agent_event = Event::new(
            "agent:alice",
            EventCategory::Agent,
            EventSeverity::Info,
            EventData::Text("test".into()),
        );

        let transport_event = Event::new(
            "transport:nats",
            EventCategory::Transport,
            EventSeverity::Info,
            EventData::Text("test".into()),
        );

        assert!(filter.should_capture(&agent_event));
        assert!(!filter.should_capture(&transport_event));
    }

    #[test]
    fn test_and_filter() {
        let severity_filter = Arc::new(SeverityFilter::new(EventSeverity::Warn));
        let category_filter = Arc::new(CategoryFilter::new(vec![EventCategory::System]));
        let filter = AndFilter::new(vec![severity_filter, category_filter]);

        // System + Warn = pass
        let pass_event = Event::new(
            "test",
            EventCategory::System,
            EventSeverity::Warn,
            EventData::Text("test".into()),
        );

        // System + Info = fail (severity too low)
        let fail_event1 = Event::new(
            "test",
            EventCategory::System,
            EventSeverity::Info,
            EventData::Text("test".into()),
        );

        // Agent + Warn = fail (wrong category)
        let fail_event2 = Event::new(
            "test",
            EventCategory::Agent,
            EventSeverity::Warn,
            EventData::Text("test".into()),
        );

        assert!(filter.should_capture(&pass_event));
        assert!(!filter.should_capture(&fail_event1));
        assert!(!filter.should_capture(&fail_event2));
    }

    #[test]
    fn test_or_filter() {
        let severity_filter = Arc::new(SeverityFilter::new(EventSeverity::Error));
        let category_filter = Arc::new(CategoryFilter::new(vec![EventCategory::Agent]));
        let filter = OrFilter::new(vec![severity_filter, category_filter]);

        // Agent + Info = pass (category matches)
        let pass_event1 = Event::new(
            "test",
            EventCategory::Agent,
            EventSeverity::Info,
            EventData::Text("test".into()),
        );

        // System + Error = pass (severity matches)
        let pass_event2 = Event::new(
            "test",
            EventCategory::System,
            EventSeverity::Error,
            EventData::Text("test".into()),
        );

        // System + Info = fail (neither matches)
        let fail_event = Event::new(
            "test",
            EventCategory::System,
            EventSeverity::Info,
            EventData::Text("test".into()),
        );

        assert!(filter.should_capture(&pass_event1));
        assert!(filter.should_capture(&pass_event2));
        assert!(!filter.should_capture(&fail_event));
    }

    #[test]
    fn test_not_filter() {
        let severity_filter = Arc::new(SeverityFilter::new(EventSeverity::Error));
        let filter = NotFilter::new(severity_filter);

        let info_event = Event::new(
            "test",
            EventCategory::System,
            EventSeverity::Info,
            EventData::Text("test".into()),
        );

        let error_event = Event::new(
            "test",
            EventCategory::System,
            EventSeverity::Error,
            EventData::Text("test".into()),
        );

        assert!(filter.should_capture(&info_event));
        assert!(!filter.should_capture(&error_event));
    }

    #[test]
    fn test_category_filter_builder() {
        let filter = CategoryFilter::new(vec![EventCategory::Message])
            .allow(EventCategory::Agent);

        let message_event = Event::new(
            "test",
            EventCategory::Message,
            EventSeverity::Info,
            EventData::Text("test".into()),
        );

        let agent_event = Event::new(
            "test",
            EventCategory::Agent,
            EventSeverity::Info,
            EventData::Text("test".into()),
        );

        let system_event = Event::new(
            "test",
            EventCategory::System,
            EventSeverity::Info,
            EventData::Text("test".into()),
        );

        assert!(filter.should_capture(&message_event));
        assert!(filter.should_capture(&agent_event));
        assert!(!filter.should_capture(&system_event));
    }
}
