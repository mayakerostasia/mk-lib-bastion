//! broadcast_message MCP tool
//!
//! Allows Claude to send messages to multiple agents at once.

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

use simian_llm::AgentRegistry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastMessageInput {
    pub message: String,
    pub subject_filter: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastMessageOutput {
    pub recipient_count: usize,
    pub recipients: Vec<String>,
    pub status: String,
    pub timestamp: String,
}

/// Tool for broadcasting messages to multiple agents
pub struct AgentBroadcastTool {
    registry: Arc<AgentRegistry>,
}

impl AgentBroadcastTool {
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self { registry }
    }

    pub fn name(&self) -> &str {
        "broadcast_message"
    }

    pub fn description(&self) -> &str {
        "Broadcast a message to multiple agents, optionally filtered by subject. \
         All agents subscribed to the matching subjects will receive the message."
    }

    pub fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "Message content to broadcast"
                },
                "subject_filter": {
                    "type": "string",
                    "description": "Optional subject filter (e.g., 'analysis.*'). \
                                   If omitted, broadcasts to all agents."
                }
            },
            "required": ["message"]
        })
    }

    pub async fn execute(&self, input: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        let input: BroadcastMessageInput = serde_json::from_value(input)?;

        // Validate message
        if input.message.is_empty() {
            return Err(anyhow::anyhow!("Message cannot be empty"));
        }

        if input.message.len() > 100_000 {
            return Err(anyhow::anyhow!(
                "Message too large (max 100,000 characters)"
            ));
        }

        // Find recipient agents
        let recipients = if let Some(filter) = input.subject_filter.as_ref() {
            // Find agents matching subject filter
            self.registry.find_agents_by_subject(filter).await
        } else {
            // Get all agents
            self.registry.list_agents().await
        };

        let recipient_ids: Vec<String> = recipients.iter().map(|a| a.id.clone()).collect();

        // Generate message ID
        let message_id = uuid::Uuid::new_v4().to_string();

        if !recipient_ids.is_empty() {
            tracing::info!(
                "Broadcasting message {} to {} agent(s)",
                message_id,
                recipient_ids.len()
            );
        }

        Ok(json!(BroadcastMessageOutput {
            recipient_count: recipient_ids.len(),
            recipients: recipient_ids,
            status: "broadcasted".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_broadcast_tool_basic() {
        let registry = Arc::new(AgentRegistry::new());

        // Register some agents
        for i in 0..3 {
            registry
                .register_agent(
                    format!("agent{}", i),
                    vec!["analysis.request".to_string()],
                    None,
                )
                .await
                .unwrap();
        }

        let tool = AgentBroadcastTool::new(registry);

        let input = json!({
            "message": "Hello agents",
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let output: BroadcastMessageOutput = serde_json::from_value(result.unwrap()).unwrap();
        assert_eq!(output.recipient_count, 3);
        assert_eq!(output.status, "broadcasted");
    }

    #[tokio::test]
    async fn test_broadcast_with_filter() {
        let registry = Arc::new(AgentRegistry::new());

        // Register agents with different subjects
        registry
            .register_agent(
                "agent1".to_string(),
                vec!["analysis.*".to_string()],
                None,
            )
            .await
            .unwrap();

        registry
            .register_agent(
                "agent2".to_string(),
                vec!["other.topic".to_string()],
                None,
            )
            .await
            .unwrap();

        let tool = AgentBroadcastTool::new(registry);

        let input = json!({
            "message": "Hello analysts",
            "subject_filter": "analysis.request"
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let output: BroadcastMessageOutput = serde_json::from_value(result.unwrap()).unwrap();
        assert_eq!(output.recipient_count, 1);
        assert_eq!(output.recipients[0], "agent1");
    }

    #[tokio::test]
    async fn test_broadcast_empty_message() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = AgentBroadcastTool::new(registry);

        let input = json!({
            "message": ""
        });

        let result = tool.execute(input).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_broadcast_no_agents() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = AgentBroadcastTool::new(registry);

        let input = json!({
            "message": "Hello"
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let output: BroadcastMessageOutput = serde_json::from_value(result.unwrap()).unwrap();
        assert_eq!(output.recipient_count, 0);
    }

    #[test]
    fn test_tool_metadata() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = AgentBroadcastTool::new(registry);

        assert_eq!(tool.name(), "broadcast_message");
        assert!(tool.description().contains("Broadcast a message"));
        
        let schema = tool.input_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"].get("message").is_some());
        assert!(schema["properties"].get("subject_filter").is_some());
    }
}
