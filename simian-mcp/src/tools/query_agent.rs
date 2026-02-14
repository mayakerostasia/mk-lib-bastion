//! query_agent_state MCP tool
//!
//! Allows Claude to get detailed information about a specific agent.

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

use simian_llm::AgentRegistry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryAgentInput {
    pub agent_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryAgentOutput {
    pub id: String,
    pub subjects: Vec<String>,
    pub status: String,
    pub system_prompt: Option<String>,
    pub last_heartbeat: String,
}

/// Tool for querying agent state
pub struct AgentQueryTool {
    registry: Arc<AgentRegistry>,
}

impl AgentQueryTool {
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self { registry }
    }

    pub fn name(&self) -> &str {
        "query_agent_state"
    }

    pub fn description(&self) -> &str {
        "Get detailed information about a specific agent, including its subjects, \
         status, and system prompt. Use this to check agent capabilities before delegating work."
    }

    pub fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "agent_id": {
                    "type": "string",
                    "description": "ID of the agent to query"
                }
            },
            "required": ["agent_id"]
        })
    }

    pub async fn execute(&self, input: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        let input: QueryAgentInput = serde_json::from_value(input)?;

        // Validate agent ID
        if input.agent_id.is_empty() {
            return Err(anyhow::anyhow!("Agent ID cannot be empty"));
        }

        // Get agent from registry
        let agent = self.registry.get_agent(&input.agent_id).await
            .ok_or_else(|| anyhow::anyhow!("Agent {} not found", &input.agent_id))?;

        Ok(json!(QueryAgentOutput {
            id: agent.id.clone(),
            subjects: agent.subjects.clone(),
            status: format!("{:?}", agent.status),
            system_prompt: agent.system_prompt.clone(),
            last_heartbeat: agent
                .last_heartbeat
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .to_string(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_query_agent_tool() {
        let registry = Arc::new(AgentRegistry::new());

        // Register an agent
        registry
            .register_agent(
                "test_agent".to_string(),
                vec!["analysis.request".to_string(), "analysis.*".to_string()],
                Some("You are an analysis agent".to_string()),
            )
            .await
            .unwrap();

        let tool = AgentQueryTool::new(registry);

        let input = json!({
            "agent_id": "test_agent"
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let output: QueryAgentOutput = serde_json::from_value(result.unwrap()).unwrap();
        assert_eq!(output.id, "test_agent");
        assert_eq!(output.subjects.len(), 2);
        assert!(output.system_prompt.is_some());
    }

    #[tokio::test]
    async fn test_query_agent_not_found() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = AgentQueryTool::new(registry);

        let input = json!({
            "agent_id": "nonexistent"
        });

        let result = tool.execute(input).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_query_agent_empty_id() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = AgentQueryTool::new(registry);

        let input = json!({
            "agent_id": ""
        });

        let result = tool.execute(input).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_tool_metadata() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = AgentQueryTool::new(registry);

        assert_eq!(tool.name(), "query_agent_state");
        assert!(tool.description().contains("Get detailed information"));
        
        let schema = tool.input_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"].get("agent_id").is_some());
        assert!(schema["required"].as_array().unwrap().contains(&json!("agent_id")));
    }
}
