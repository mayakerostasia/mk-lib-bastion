//! Agent spawning and lifecycle management
//!
//! The `AgentSpawner` enables creating new agents dynamically with custom system prompts
//! and subject subscriptions. Supports both ephemeral agents (with task join handles) and
//! persistent agents (fire-and-forget).
//!
//! # Design Philosophy
//!
//! - **Ephemeral agents**: Return a JoinHandle for the task, allowing caller to wait/cancel
//! - **Persistent agents**: Fire-and-forget, no handle returned
//! - **Auto-ID generation**: Generate UUIDs if agent_id not provided
//! - **Subject flexibility**: Agents choose their own subjects at spawn time

use crate::registry::{AgentRegistry, AgentStatus};
use uuid::Uuid;

/// Configuration for spawning a new agent
#[derive(Debug, Clone)]
pub struct SpawnConfig {
    /// Agent ID (auto-generated if None)
    pub agent_id: Option<String>,
    /// System prompt for the agent
    pub system_prompt: Option<String>,
    /// Subjects the agent listens to
    pub subjects: Vec<String>,
    /// Whether agent should be persistent (true) or ephemeral (false)
    pub persistent: bool,
}

impl SpawnConfig {
    /// Create a new spawn config builder
    pub fn new() -> Self {
        Self {
            agent_id: None,
            system_prompt: None,
            subjects: Vec::new(),
            persistent: false,
        }
    }

    /// Set the agent ID
    pub fn with_id(mut self, id: String) -> Self {
        self.agent_id = Some(id);
        self
    }

    /// Set the system prompt
    pub fn with_system_prompt(mut self, prompt: String) -> Self {
        self.system_prompt = Some(prompt);
        self
    }

    /// Add a subject
    pub fn with_subject(mut self, subject: String) -> Self {
        self.subjects.push(subject);
        self
    }

    /// Add multiple subjects
    pub fn with_subjects(mut self, subjects: Vec<String>) -> Self {
        self.subjects.extend(subjects);
        self
    }

    /// Set persistence mode
    pub fn persistent(mut self, persistent: bool) -> Self {
        self.persistent = persistent;
        self
    }

    /// Get or generate agent ID
    pub fn get_or_generate_id(&self) -> String {
        self.agent_id
            .clone()
            .unwrap_or_else(|| format!("agent-{}", Uuid::new_v4()))
    }
}

impl Default for SpawnConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle to an agent task
#[derive(Debug)]
pub enum AgentHandle {
    /// Ephemeral agent with task join handle
    Ephemeral(tokio::task::JoinHandle<()>),
    /// Persistent agent (no handle)
    Persistent(String),
}

impl AgentHandle {
    /// Get the agent ID
    pub fn agent_id(&self) -> &str {
        match self {
            AgentHandle::Ephemeral(_) => "unknown", // Would need to track ID separately
            AgentHandle::Persistent(id) => id,
        }
    }

    /// Check if this is a persistent agent
    pub fn is_persistent(&self) -> bool {
        matches!(self, AgentHandle::Persistent(_))
    }

    /// Check if this is an ephemeral agent
    pub fn is_ephemeral(&self) -> bool {
        matches!(self, AgentHandle::Ephemeral(_))
    }
}

/// Spawns new agents and manages their lifecycle
pub struct AgentSpawner {
    registry: AgentRegistry,
}

impl AgentSpawner {
    /// Create a new spawner with the given registry
    pub fn new(registry: AgentRegistry) -> Self {
        Self { registry }
    }

    /// Spawn a new agent with the given configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Spawn configuration
    /// * `task` - Async task for the agent to run
    ///
    /// # Returns
    ///
    /// Returns `AgentHandle` which allows tracking ephemeral agents
    pub async fn spawn<F>(&self, config: SpawnConfig, task: F) -> Result<AgentHandle, String>
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let agent_id = config.get_or_generate_id();
        let system_prompt = config
            .system_prompt
            .as_ref()
            .map(|p| p.to_string());

        // Register agent in registry
        self.registry
            .register_agent(agent_id.clone(), config.subjects.clone(), system_prompt)
            .await?;

        // Update status to running
        self.registry
            .update_status(&agent_id, AgentStatus::Running)
            .await?;

        let handle = if config.persistent {
            // Fire-and-forget for persistent agents
            tokio::spawn(task);
            AgentHandle::Persistent(agent_id)
        } else {
            // Return handle for ephemeral agents
            let handle = tokio::spawn(task);
            AgentHandle::Ephemeral(handle)
        };

        Ok(handle)
    }

    /// Get reference to the underlying registry
    pub fn registry(&self) -> &AgentRegistry {
        &self.registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_config_default() {
        let config = SpawnConfig::new();

        assert!(config.agent_id.is_none());
        assert!(config.system_prompt.is_none());
        assert!(config.subjects.is_empty());
        assert!(!config.persistent);
    }

    #[test]
    fn test_spawn_config_builder() {
        let config = SpawnConfig::new()
            .with_id("my-agent".to_string())
            .with_system_prompt("You are helpful".to_string())
            .with_subject("analysis.request".to_string())
            .persistent(true);

        assert_eq!(config.agent_id, Some("my-agent".to_string()));
        assert_eq!(config.system_prompt, Some("You are helpful".to_string()));
        assert_eq!(config.subjects, vec!["analysis.request".to_string()]);
        assert!(config.persistent);
    }

    #[test]
    fn test_spawn_config_multiple_subjects() {
        let config = SpawnConfig::new()
            .with_subjects(vec![
                "analysis.request".to_string(),
                "analysis.query".to_string(),
            ])
            .with_subject("analysis.response".to_string());

        assert_eq!(config.subjects.len(), 3);
    }

    #[test]
    fn test_get_or_generate_id_with_id() {
        let config = SpawnConfig::new().with_id("my-agent".to_string());

        assert_eq!(config.get_or_generate_id(), "my-agent");
    }

    #[test]
    fn test_get_or_generate_id_without_id() {
        let config = SpawnConfig::new();
        let id = config.get_or_generate_id();

        assert!(id.starts_with("agent-"));
        assert!(id.len() > "agent-".len()); // Contains UUID
    }

    #[test]
    fn test_agent_handle_persistent() {
        let handle = AgentHandle::Persistent("agent-1".to_string());

        assert!(handle.is_persistent());
        assert!(!handle.is_ephemeral());
        assert_eq!(handle.agent_id(), "agent-1");
    }

    #[tokio::test]
    async fn test_spawner_creation() {
        let registry = AgentRegistry::new();
        let spawner = AgentSpawner::new(registry.clone());

        assert_eq!(spawner.registry().agent_count().await, 0);
    }

    #[tokio::test]
    async fn test_spawn_persistent_agent() {
        let registry = AgentRegistry::new();
        let spawner = AgentSpawner::new(registry.clone());

        let config = SpawnConfig::new()
            .with_id("test-agent".to_string())
            .with_subject("test.subject".to_string())
            .persistent(true);

        let result = spawner
            .spawn(config, async {
                // Simulate agent work
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            })
            .await;

        assert!(result.is_ok());
        let handle = result.unwrap();
        assert!(handle.is_persistent());

        // Agent should be registered
        assert_eq!(registry.agent_count().await, 1);
        let agent = registry.get_agent("test-agent").await.unwrap();
        assert_eq!(agent.status, AgentStatus::Running);
    }

    #[tokio::test]
    async fn test_spawn_ephemeral_agent() {
        let registry = AgentRegistry::new();
        let spawner = AgentSpawner::new(registry.clone());

        let config = SpawnConfig::new()
            .with_id("ephemeral-agent".to_string())
            .with_subject("test.subject".to_string())
            .persistent(false);

        let result = spawner
            .spawn(config, async {
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            })
            .await;

        assert!(result.is_ok());
        let handle = result.unwrap();
        assert!(handle.is_ephemeral());

        // Agent should be registered
        assert_eq!(registry.agent_count().await, 1);
    }

    #[tokio::test]
    async fn test_spawn_duplicate_id() {
        let registry = AgentRegistry::new();
        let spawner = AgentSpawner::new(registry.clone());

        let config = SpawnConfig::new().with_id("agent-1".to_string());

        let result1 = spawner
            .spawn(config.clone(), async { tokio::time::sleep(tokio::time::Duration::from_secs(10)).await })
            .await;

        assert!(result1.is_ok());

        let result2 = spawner
            .spawn(config, async { tokio::time::sleep(tokio::time::Duration::from_secs(10)).await })
            .await;

        assert!(result2.is_err());
    }

    #[tokio::test]
    async fn test_spawn_with_system_prompt() {
        let registry = AgentRegistry::new();
        let spawner = AgentSpawner::new(registry.clone());

        let config = SpawnConfig::new()
            .with_id("analyst".to_string())
            .with_system_prompt("You are a data analyst".to_string())
            .persistent(true);

        let _ = spawner
            .spawn(config, async {})
            .await;

        let agent = registry.get_agent("analyst").await.unwrap();
        assert_eq!(agent.system_prompt, Some("You are a data analyst".to_string()));
    }

    #[tokio::test]
    async fn test_spawn_multiple_subjects() {
        let registry = AgentRegistry::new();
        let spawner = AgentSpawner::new(registry.clone());

        let config = SpawnConfig::new()
            .with_id("multi-subject".to_string())
            .with_subjects(vec![
                "analysis.request".to_string(),
                "analysis.query".to_string(),
            ])
            .persistent(true);

        let _ = spawner
            .spawn(config, async {})
            .await;

        let agent = registry.get_agent("multi-subject").await.unwrap();
        assert_eq!(agent.subjects.len(), 2);
    }
}
