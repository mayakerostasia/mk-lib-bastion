//! list_agents MCP tool
//!
//! Allows Claude to query available agents in the registry.

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

use simian_llm::AgentRegistry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListAgentsInput {
    pub filter: Option<String>, // subject or status filter
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub subjects: Vec<String>,
    pub status: String,
    pub system_prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListAgentsOutput {
    pub agents: Vec<AgentInfo>,
    pub total_count: usize,
}

/// Tool for listing and discovering agents
pub struct AgentListTool {
    registry: Arc<AgentRegistry>,
}

impl AgentListTool {
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self { registry }
    }

    pub fn name(&self) -> &str {
        "list_agents"
    }

    pub fn description(&self) -> &str {
        "List all active agents in the registry, with optional filtering by subject. \
         Use this to discover available agents before sending messages or broadcasting."
    }

    pub fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "filter": {
                    "type": "string",
                    "description": "Optional filter - can be a subject pattern (e.g., 'analysis.*') \
                                   or status (e.g., 'running', 'stopped')"
                }
            },
            "required": []
        })
    }

    pub async fn execute(&self, input: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        let input: ListAgentsInput = serde_json::from_value(input)?;

        let agents = if let Some(filter) = input.filter.as_ref() {
            // Try to interpret filter as subject first
            let subject_matches = self.registry.find_agents_by_subject(filter).await;
            if !subject_matches.is_empty() {
                subject_matches
            } else {
                // If not a subject, try as status filter
                let all_agents = self.registry.list_agents().await;
                all_agents
                    .into_iter()
                    .filter(|a| {
                        format!("{:?}", a.status).to_lowercase()
                            == filter.to_lowercase()
                    })
                    .collect()
            }
        } else {
            // No filter, return all agents
            self.registry.list_agents().await
        };

        let agent_infos: Vec<AgentInfo> = agents
            .iter()
            .map(|a| AgentInfo {
                id: a.id.clone(),
                subjects: a.subjects.clone(),
                status: format!("{:?}", a.status),
                system_prompt: a.system_prompt.clone(),
            })
            .collect();

        let total_count = agent_infos.len();

        Ok(json!(ListAgentsOutput {
            agents: agent_infos,
            total_count,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_agents_tool() {
        let registry = Arc::new(AgentRegistry::new());

        // Register some agents
        for i in 0..3 {
            registry
                .register_agent(
                    format!("agent{}", i),
                    vec!["test.request".to_string()],
                    Some(format!("Agent {} system prompt", i)),
                )
                .await
                .unwrap();
        }

        let tool = AgentListTool::new(registry);

        let input = json!({});

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let output: ListAgentsOutput = serde_json::from_value(result.unwrap()).unwrap();
        assert_eq!(output.total_count, 3);
        assert_eq!(output.agents.len(), 3);
    }

    #[tokio::test]
    async fn test_list_agents_with_subject_filter() {
        let registry = Arc::new(AgentRegistry::new());

        // Register agents with different subjects
        registry
            .register_agent(
                "analyst".to_string(),
                vec!["analysis.*".to_string()],
                Some("Analyze data".to_string()),
            )
            .await
            .unwrap();

        registry
            .register_agent(
                "writer".to_string(),
                vec!["writing.*".to_string()],
                Some("Write reports".to_string()),
            )
            .await
            .unwrap();

        let tool = AgentListTool::new(registry);

        let input = json!({
            "filter": "analysis.request"
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let output: ListAgentsOutput = serde_json::from_value(result.unwrap()).unwrap();
        assert_eq!(output.total_count, 1);
        assert_eq!(output.agents[0].id, "analyst");
    }

    #[tokio::test]
    async fn test_list_agents_empty() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = AgentListTool::new(registry);

        let input = json!({});

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let output: ListAgentsOutput = serde_json::from_value(result.unwrap()).unwrap();
        assert_eq!(output.total_count, 0);
        assert!(output.agents.is_empty());
    }

    #[test]
    fn test_tool_metadata() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = AgentListTool::new(registry);

        assert_eq!(tool.name(), "list_agents");
        assert!(tool.description().contains("List all active agents"));
        
        let schema = tool.input_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"].get("filter").is_some());
    }
}
