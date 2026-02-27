use chrono::Utc;
use simian_base_api::transport::AgentId;
use simian_llm::agent::AgentAnnouncement;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub use simian_llm::agent::{AgentMetadata, AgentQuery, AgentQueryResponse};

/// In-memory agent registry.
///
/// Thread-safe via interior mutability — clone freely to share across tasks.
#[derive(Clone, Debug, Default)]
pub struct AgentRegistry {
    entries: Arc<Mutex<HashMap<String, AgentMetadata>>>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers or refreshes an agent from an announcement.
    pub fn register(&self, announcement: &AgentAnnouncement) {
        let mut map = self.entries.lock().unwrap();
        map.insert(
            announcement.agent_id.clone(),
            AgentMetadata {
                agent_id: AgentId::new(&announcement.agent_id),
                subjects: announcement.subjects.clone(),
                capabilities: announcement.capabilities.clone(),
                last_seen: Utc::now().timestamp(),
            },
        );
    }

    /// Updates the `last_seen` timestamp for a known agent (heartbeat).
    /// Silently ignores unknown agents — they should re-register.
    pub fn heartbeat(&self, agent_id: &str) {
        let mut map = self.entries.lock().unwrap();
        if let Some(entry) = map.get_mut(agent_id) {
            entry.last_seen = Utc::now().timestamp();
        }
    }

    /// Returns agents matching the query filters.
    pub fn query(&self, q: &AgentQuery) -> Vec<AgentMetadata> {
        let map = self.entries.lock().unwrap();
        map.values()
            .filter(|m| {
                let cap_ok = q.capability.as_deref().map_or(true, |cap| {
                    m.capabilities.iter().any(|c| c == cap)
                });
                let subj_ok = q.subject.as_deref().map_or(true, |target| {
                    m.subjects.iter().any(|pattern| {
                        simian_base_api::transport::match_subject(pattern, target)
                    })
                });
                cap_ok && subj_ok
            })
            .cloned()
            .collect()
    }

    /// Removes agents whose last heartbeat is older than `max_age_secs`.
    /// Returns the number of entries removed.
    pub fn remove_stale(&self, max_age_secs: i64) -> usize {
        let cutoff = Utc::now().timestamp() - max_age_secs;
        let mut map = self.entries.lock().unwrap();
        let before = map.len();
        map.retain(|_, m| m.last_seen >= cutoff);
        before - map.len()
    }

    /// Returns a snapshot of all currently registered agents.
    pub fn all(&self) -> Vec<AgentMetadata> {
        self.entries.lock().unwrap().values().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.entries.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.lock().unwrap().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn announcement(id: &str, subjects: &[&str], caps: &[&str]) -> AgentAnnouncement {
        AgentAnnouncement {
            agent_id: id.to_string(),
            subjects: subjects.iter().map(|s| s.to_string()).collect(),
            capabilities: caps.iter().map(|c| c.to_string()).collect(),
            timestamp: Utc::now().timestamp(),
        }
    }

    #[test]
    fn test_register_and_query_by_capability() {
        let registry = AgentRegistry::new();
        registry.register(&announcement("agent-a", &["analysis.*"], &["text-analysis"]));
        registry.register(&announcement("agent-b", &["data.*"], &["data-import"]));

        let results = registry.query(&AgentQuery {
            capability: Some("text-analysis".to_string()),
            subject: None,
        });
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].agent_id.0, "agent-a");
    }

    #[test]
    fn test_query_by_subject_wildcard() {
        let registry = AgentRegistry::new();
        registry.register(&announcement("agent-a", &["data.*"], &["processor"]));

        let results = registry.query(&AgentQuery {
            capability: None,
            subject: Some("data.import".to_string()),
        });
        assert_eq!(results.len(), 1);

        let no_results = registry.query(&AgentQuery {
            capability: None,
            subject: Some("data.import.csv".to_string()),
        });
        assert!(no_results.is_empty());
    }

    #[test]
    fn test_remove_stale() {
        let registry = AgentRegistry::new();
        // Insert an entry with an old last_seen
        {
            let mut map = registry.entries.lock().unwrap();
            map.insert(
                "old-agent".to_string(),
                AgentMetadata {
                    agent_id: AgentId::new("old-agent"),
                    subjects: vec![],
                    capabilities: vec![],
                    last_seen: Utc::now().timestamp() - 200,
                },
            );
        }
        registry.register(&announcement("fresh-agent", &[], &[]));

        let removed = registry.remove_stale(90);
        assert_eq!(removed, 1);
        assert_eq!(registry.len(), 1);
        assert_eq!(registry.all()[0].agent_id.0, "fresh-agent");
    }

    #[test]
    fn test_heartbeat_refreshes_last_seen() {
        let registry = AgentRegistry::new();
        {
            let mut map = registry.entries.lock().unwrap();
            map.insert(
                "agent-x".to_string(),
                AgentMetadata {
                    agent_id: AgentId::new("agent-x"),
                    subjects: vec![],
                    capabilities: vec![],
                    last_seen: Utc::now().timestamp() - 200,
                },
            );
        }

        registry.heartbeat("agent-x");

        let removed = registry.remove_stale(90);
        assert_eq!(removed, 0, "heartbeat should have refreshed last_seen");
    }

    #[test]
    fn test_query_and_filter() {
        let registry = AgentRegistry::new();
        registry.register(&announcement("a", &["llm.*"], &["chat", "summarise"]));
        registry.register(&announcement("b", &["data.*"], &["ingest"]));

        // Both filters: only "a" has capability "chat" AND handles "llm.request"
        let results = registry.query(&AgentQuery {
            capability: Some("chat".to_string()),
            subject: Some("llm.request".to_string()),
        });
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].agent_id.0, "a");
    }
}
