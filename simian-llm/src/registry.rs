//! Agent Registry for discovering and managing active agents
//!
//! The `AgentRegistry` provides a central discovery system for all active agents in the system.
//! It tracks agent metadata, allows querying agents by subject or capability, and maintains
//! agent status information.
//!
//! # Design Philosophy
//!
//! - **Global singleton**: Accessed via Arc for thread-safe, cheap cloning
//! - **Subject-based discovery**: Agents can be found by subjects they listen to
//! - **Capability search**: Find agents by their specialization/capabilities
//! - **Status tracking**: Monitor agent health and lifecycle state

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Represents the lifecycle state of an agent
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStatus {
    /// Agent is starting up
    Starting,
    /// Agent is active and listening
    Running,
    /// Agent is shutting down
    Stopping,
    /// Agent has stopped
    Stopped,
    /// Agent encountered an error
    Error,
}

/// Metadata about an active agent
#[derive(Debug, Clone)]
pub struct AgentMetadata {
    /// Unique identifier for this agent
    pub id: String,
    /// Subjects this agent listens to
    pub subjects: Vec<String>,
    /// Current lifecycle status
    pub status: AgentStatus,
    /// When the agent last sent a heartbeat
    pub last_heartbeat: std::time::SystemTime,
    /// Optional system prompt identifying the agent's role
    pub system_prompt: Option<String>,
}

/// Central registry for discovering and managing agents
///
/// Thread-safe via Arc<RwLock<>>. Supports concurrent reads and exclusive writes.
#[derive(Clone)]
pub struct AgentRegistry {
    agents: Arc<RwLock<HashMap<String, AgentMetadata>>>,
}

impl AgentRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new agent with metadata
    pub async fn register_agent(
        &self,
        id: String,
        subjects: Vec<String>,
        system_prompt: Option<String>,
    ) -> Result<(), String> {
        let mut agents = self.agents.write().await;

        if agents.contains_key(&id) {
            return Err(format!("Agent {} already registered", id));
        }

        let metadata = AgentMetadata {
            id: id.clone(),
            subjects,
            status: AgentStatus::Starting,
            last_heartbeat: std::time::SystemTime::now(),
            system_prompt,
        };

        agents.insert(id, metadata);
        Ok(())
    }

    /// Unregister an agent from the registry
    pub async fn unregister_agent(&self, id: &str) -> Result<(), String> {
        let mut agents = self.agents.write().await;

        if agents.remove(id).is_none() {
            return Err(format!("Agent {} not found", id));
        }

        Ok(())
    }

    /// Get metadata for a specific agent
    pub async fn get_agent(&self, id: &str) -> Option<AgentMetadata> {
        self.agents.read().await.get(id).cloned()
    }

    /// Update agent status
    pub async fn update_status(&self, id: &str, status: AgentStatus) -> Result<(), String> {
        let mut agents = self.agents.write().await;

        let metadata = agents.get_mut(id).ok_or_else(|| format!("Agent {} not found", id))?;

        metadata.status = status;
        metadata.last_heartbeat = std::time::SystemTime::now();

        Ok(())
    }

    /// Find all agents listening to a specific subject
    ///
    /// Supports both exact and wildcard matching:
    /// - Exact: "analysis.request" matches agent subscribed to "analysis.request"
    /// - Wildcard: "data.import" matches agent subscribed to "data.*"
    pub async fn find_agents_by_subject(&self, subject: &str) -> Vec<AgentMetadata> {
        let agents = self.agents.read().await;

        agents
            .values()
            .filter(|metadata| self.subject_matches(&metadata.subjects, subject))
            .cloned()
            .collect()
    }

    /// Check if a subject matches any subscribed subjects (with wildcard support)
    fn subject_matches(&self, subscriptions: &[String], subject: &str) -> bool {
        subscriptions.iter().any(|sub| {
            // Exact match
            if sub == subject {
                return true;
            }

            // Wildcard match: "a.*" matches "a.b" but NOT "a.b.c"
            // Pattern: prefix.* matches prefix.X where X has no more dots
            if let Some(prefix) = sub.strip_suffix(".*") {
                if subject.starts_with(prefix) && subject[prefix.len()..].starts_with('.') {
                    // Get the part after the dot
                    let after_dot = &subject[prefix.len() + 1..];
                    // Should not contain another dot
                    return !after_dot.contains('.');
                }
            }

            false
        })
    }

    /// Find agents by capability (searches system prompts for keywords)
    pub async fn find_agents_by_capability(&self, capability: &str) -> Vec<AgentMetadata> {
        let agents = self.agents.read().await;

        agents
            .values()
            .filter(|metadata| {
                if let Some(prompt) = &metadata.system_prompt {
                    prompt.to_lowercase().contains(&capability.to_lowercase())
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }

    /// List all registered agents
    pub async fn list_agents(&self) -> Vec<AgentMetadata> {
        self.agents.read().await.values().cloned().collect()
    }

    /// Get count of active agents
    pub async fn agent_count(&self) -> usize {
        self.agents.read().await.len()
    }

    /// Clear all agents (for testing)
    #[cfg(test)]
    pub async fn clear(&self) {
        self.agents.write().await.clear();
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_agent() {
        let registry = AgentRegistry::new();

        let result = registry
            .register_agent(
                "agent-1".to_string(),
                vec!["analysis.request".to_string()],
                Some("You are an analyzer".to_string()),
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(registry.agent_count().await, 1);
    }

    #[tokio::test]
    async fn test_duplicate_registration() {
        let registry = AgentRegistry::new();

        let _ = registry
            .register_agent(
                "agent-1".to_string(),
                vec!["analysis.request".to_string()],
                None,
            )
            .await;

        let result = registry
            .register_agent(
                "agent-1".to_string(),
                vec!["analysis.request".to_string()],
                None,
            )
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_unregister_agent() {
        let registry = AgentRegistry::new();

        registry
            .register_agent("agent-1".to_string(), vec![], None)
            .await
            .unwrap();

        assert_eq!(registry.agent_count().await, 1);

        let result = registry.unregister_agent("agent-1").await;

        assert!(result.is_ok());
        assert_eq!(registry.agent_count().await, 0);
    }

    #[tokio::test]
    async fn test_unregister_nonexistent() {
        let registry = AgentRegistry::new();

        let result = registry.unregister_agent("nonexistent").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_agent() {
        let registry = AgentRegistry::new();

        registry
            .register_agent(
                "agent-1".to_string(),
                vec!["test.subject".to_string()],
                Some("Test agent".to_string()),
            )
            .await
            .unwrap();

        let metadata = registry.get_agent("agent-1").await;

        assert!(metadata.is_some());
        let m = metadata.unwrap();
        assert_eq!(m.id, "agent-1");
        assert_eq!(m.subjects, vec!["test.subject".to_string()]);
        assert!(m.system_prompt.is_some());
    }

    #[tokio::test]
    async fn test_update_status() {
        let registry = AgentRegistry::new();

        registry
            .register_agent("agent-1".to_string(), vec![], None)
            .await
            .unwrap();

        let result = registry
            .update_status("agent-1", AgentStatus::Running)
            .await;

        assert!(result.is_ok());

        let metadata = registry.get_agent("agent-1").await.unwrap();
        assert_eq!(metadata.status, AgentStatus::Running);
    }

    #[tokio::test]
    async fn test_find_by_subject_exact() {
        let registry = AgentRegistry::new();

        registry
            .register_agent(
                "analyzer".to_string(),
                vec!["analysis.request".to_string()],
                None,
            )
            .await
            .unwrap();

        registry
            .register_agent(
                "writer".to_string(),
                vec!["writing.request".to_string()],
                None,
            )
            .await
            .unwrap();

        let results = registry.find_agents_by_subject("analysis.request").await;

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "analyzer");
    }

    #[tokio::test]
    async fn test_find_by_subject_wildcard() {
        let registry = AgentRegistry::new();

        registry
            .register_agent(
                "analyzer".to_string(),
                vec!["data.*".to_string()],
                None,
            )
            .await
            .unwrap();

        let results = registry.find_agents_by_subject("data.import").await;
        assert_eq!(results.len(), 1);

        let results = registry.find_agents_by_subject("data.export").await;
        assert_eq!(results.len(), 1);

        let results = registry.find_agents_by_subject("other.import").await;
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_wildcard_no_recursive() {
        let registry = AgentRegistry::new();

        registry
            .register_agent("agent".to_string(), vec!["a.*".to_string()], None)
            .await
            .unwrap();

        // Should match "a.b"
        assert_eq!(registry.find_agents_by_subject("a.b").await.len(), 1);

        // Should NOT match "a.b.c" (not recursive)
        assert_eq!(registry.find_agents_by_subject("a.b.c").await.len(), 0);
    }

    #[tokio::test]
    async fn test_find_by_capability() {
        let registry = AgentRegistry::new();

        registry
            .register_agent(
                "analyzer".to_string(),
                vec![],
                Some("You are an expert data analyst".to_string()),
            )
            .await
            .unwrap();

        registry
            .register_agent(
                "writer".to_string(),
                vec![],
                Some("You are a technical writer".to_string()),
            )
            .await
            .unwrap();

        let results = registry.find_agents_by_capability("analyst").await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "analyzer");

        let results = registry.find_agents_by_capability("writer").await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "writer");
    }

    #[tokio::test]
    async fn test_list_agents() {
        let registry = AgentRegistry::new();

        registry
            .register_agent("agent-1".to_string(), vec![], None)
            .await
            .unwrap();

        registry
            .register_agent("agent-2".to_string(), vec![], None)
            .await
            .unwrap();

        let agents = registry.list_agents().await;

        assert_eq!(agents.len(), 2);
        let ids: Vec<_> = agents.iter().map(|a| a.id.clone()).collect();
        assert!(ids.contains(&"agent-1".to_string()));
        assert!(ids.contains(&"agent-2".to_string()));
    }

    #[tokio::test]
    async fn test_multiple_subjects() {
        let registry = AgentRegistry::new();

        registry
            .register_agent(
                "multi".to_string(),
                vec![
                    "analysis.request".to_string(),
                    "analysis.query".to_string(),
                ],
                None,
            )
            .await
            .unwrap();

        assert_eq!(registry.find_agents_by_subject("analysis.request").await.len(), 1);
        assert_eq!(registry.find_agents_by_subject("analysis.query").await.len(), 1);
        assert_eq!(
            registry
                .find_agents_by_subject("analysis.response")
                .await
                .len(),
            0
        );
    }

    #[tokio::test]
    async fn test_agent_count() {
        let registry = AgentRegistry::new();

        assert_eq!(registry.agent_count().await, 0);

        registry
            .register_agent("agent-1".to_string(), vec![], None)
            .await
            .unwrap();

        assert_eq!(registry.agent_count().await, 1);

        registry
            .register_agent("agent-2".to_string(), vec![], None)
            .await
            .unwrap();

        assert_eq!(registry.agent_count().await, 2);

        registry.unregister_agent("agent-1").await.unwrap();

        assert_eq!(registry.agent_count().await, 1);
    }

    #[tokio::test]
    async fn test_capability_case_insensitive() {
        let registry = AgentRegistry::new();

        registry
            .register_agent(
                "agent".to_string(),
                vec![],
                Some("You are a Writer".to_string()),
            )
            .await
            .unwrap();

        let results = registry.find_agents_by_capability("writer").await;
        assert_eq!(results.len(), 1);

        let results = registry.find_agents_by_capability("WRITER").await;
        assert_eq!(results.len(), 1);
    }
}
