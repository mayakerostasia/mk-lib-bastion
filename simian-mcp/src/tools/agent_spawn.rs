//! spawn_agent MCP tool
//!
//! Allows Claude to spawn new agents with custom system prompts, subjects, and models.

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

use simian_llm::AgentRegistry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnAgentInput {
    pub name: String,
    pub system_prompt: Option<String>,
    pub subjects: Option<Vec<String>>,
    pub mode: Option<String>, // "ephemeral" or "persistent"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnAgentOutput {
    pub agent_id: String,
    pub status: String,
    pub subjects: Vec<String>,
    pub system_prompt: Option<String>,
}

/// Tool for spawning new agents via MCP
pub struct AgentSpawnTool {
    registry: Arc<AgentRegistry>,
}

impl AgentSpawnTool {
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self { registry }
    }

    pub fn name(&self) -> &str {
        "spawn_agent"
    }

    pub fn description(&self) -> &str {
        "Create a new agent with custom system prompt, subjects, and behavior. \
         The agent will be registered in the system and can immediately start \
         accepting messages on its subscribed subjects."
    }

    pub fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Unique name/ID for the agent"
                },
                "system_prompt": {
                    "type": "string",
                    "description": "System prompt to guide the agent behavior (optional)"
                },
                "subjects": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Subjects the agent should listen on (e.g., ['analysis.request', 'data.*'])"
                },
                "mode": {
                    "type": "string",
                    "enum": ["ephemeral", "persistent"],
                    "description": "Agent lifetime mode (default: persistent)"
                }
            },
            "required": ["name"]
        })
    }

    pub async fn execute(&self, input: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        let input: SpawnAgentInput = serde_json::from_value(input)?;

        // Validate agent name
        if input.name.is_empty() {
            return Err(anyhow::anyhow!("Agent name cannot be empty"));
        }

        if input.name.len() > 255 {
            return Err(anyhow::anyhow!("Agent name too long (max 255 characters)"));
        }

        // Validate subjects
        let subjects = input.subjects.unwrap_or_default();
        if subjects.len() > 100 {
            return Err(anyhow::anyhow!("Too many subjects (max 100)"));
        }

        for subject in &subjects {
            if !Self::validate_subject(subject) {
                return Err(anyhow::anyhow!(
                    "Invalid subject '{}': must contain only alphanumeric, dots, hyphens, and wildcards",
                    subject
                ));
            }
        }

        // Validate system prompt
        if let Some(ref prompt) = input.system_prompt {
            if prompt.len() > 10000 {
                return Err(anyhow::anyhow!(
                    "System prompt too long (max 10000 characters)"
                ));
            }
        }

        let agent_id = input.name.clone();

        // Register agent in registry
        self.registry
            .register_agent(agent_id.clone(), subjects.clone(), input.system_prompt.clone())
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        Ok(json!(SpawnAgentOutput {
            agent_id,
            status: "running".to_string(),
            subjects,
            system_prompt: input.system_prompt,
        }))
    }

    fn validate_subject(subject: &str) -> bool {
        if subject.is_empty() {
            return false;
        }

        // Allow alphanumeric, dots, hyphens, underscores, and wildcards
        subject
            .chars()
            .all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_' || c == '*')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_subject_valid() {
        assert!(AgentSpawnTool::validate_subject("analysis.request"));
        assert!(AgentSpawnTool::validate_subject("data.import"));
        assert!(AgentSpawnTool::validate_subject("agents.*.inbox"));
        assert!(AgentSpawnTool::validate_subject("task_processing"));
    }

    #[test]
    fn test_validate_subject_invalid() {
        assert!(!AgentSpawnTool::validate_subject(""));
        assert!(!AgentSpawnTool::validate_subject("invalid subject")); // space
        assert!(!AgentSpawnTool::validate_subject("invalid/subject")); // slash
        assert!(!AgentSpawnTool::validate_subject("invalid@subject")); // at sign
    }

    #[tokio::test]
    async fn test_spawn_agent_tool_basic() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = AgentSpawnTool::new(registry.clone());

        let input = json!({
            "name": "test_agent",
            "system_prompt": "You are a test agent",
            "subjects": ["test.request"],
            "mode": "persistent"
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let output: SpawnAgentOutput = serde_json::from_value(result.unwrap()).unwrap();
        assert_eq!(output.agent_id, "test_agent");
        assert_eq!(output.status, "running");
        assert_eq!(output.subjects, vec!["test.request"]);
    }

    #[tokio::test]
    async fn test_spawn_agent_empty_name() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = AgentSpawnTool::new(registry);

        let input = json!({
            "name": "",
        });

        let result = tool.execute(input).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_spawn_agent_invalid_subject() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = AgentSpawnTool::new(registry);

        let input = json!({
            "name": "test_agent",
            "subjects": ["invalid subject"],
        });

        let result = tool.execute(input).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_tool_metadata() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = AgentSpawnTool::new(registry);

        assert_eq!(tool.name(), "spawn_agent");
        assert!(tool.description().contains("Create a new agent"));
        
        let schema = tool.input_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"].get("name").is_some());
        assert!(schema["properties"].get("system_prompt").is_some());
        assert!(schema["properties"].get("subjects").is_some());
    }
}
