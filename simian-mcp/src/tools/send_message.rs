//! send_message MCP tool
//!
//! Allows Claude to send messages directly to specific agents.

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

use simian_llm::AgentRegistry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageInput {
    pub agent_id: String,
    pub message: String,
    pub subject: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageOutput {
    pub agent_id: String,
    pub message_id: String,
    pub status: String,
    pub timestamp: String,
}

/// Tool for sending messages to specific agents
pub struct SendMessageTool {
    registry: Arc<AgentRegistry>,
}

impl SendMessageTool {
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self { registry }
    }

    pub fn name(&self) -> &str {
        "send_message"
    }

    pub fn description(&self) -> &str {
        "Send a message directly to a specific agent. The agent will process the message \
         and may respond with a result."
    }

    pub fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "agent_id": {
                    "type": "string",
                    "description": "ID of the target agent"
                },
                "message": {
                    "type": "string",
                    "description": "Message content to send"
                },
                "subject": {
                    "type": "string",
                    "description": "Optional subject for routing (e.g., 'analysis.request')"
                }
            },
            "required": ["agent_id", "message"]
        })
    }

    pub async fn execute(&self, input: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        let input: SendMessageInput = serde_json::from_value(input)?;

        // Validate agent ID
        if input.agent_id.is_empty() {
            return Err(anyhow::anyhow!("Agent ID cannot be empty"));
        }

        // Validate message
        if input.message.is_empty() {
            return Err(anyhow::anyhow!("Message cannot be empty"));
        }

        if input.message.len() > 100_000 {
            return Err(anyhow::anyhow!(
                "Message too large (max 100,000 characters)"
            ));
        }

        // Check if agent exists
        let agent = self.registry.get_agent(&input.agent_id).await
            .ok_or_else(|| anyhow::anyhow!("Agent {} not found", &input.agent_id))?;

        // Generate message ID
        let message_id = uuid::Uuid::new_v4().to_string();

        // If subject is provided, validate it
        if let Some(ref subject) = input.subject {
            if !agent.subjects.contains(subject) && !Self::matches_wildcard(&agent.subjects, subject) {
                tracing::warn!(
                    "Agent {} not subscribed to subject {}",
                    input.agent_id,
                    subject
                );
            }
        }

        // Return success response
        Ok(json!(SendMessageOutput {
            agent_id: input.agent_id,
            message_id,
            status: "sent".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }))
    }

    fn matches_wildcard(subjects: &[String], target: &str) -> bool {
        subjects.iter().any(|s| {
            if s.ends_with(".*") {
                let prefix = &s[..s.len() - 2];
                target.starts_with(prefix) && target[prefix.len()..].starts_with('.')
                    && !target[prefix.len() + 1..].contains('.')
            } else {
                s == target
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_wildcard() {
        let subjects = vec!["analysis.*".to_string(), "exact.match".to_string()];
        
        assert!(SendMessageTool::matches_wildcard(&subjects, "analysis.request"));
        assert!(SendMessageTool::matches_wildcard(&subjects, "exact.match"));
        assert!(!SendMessageTool::matches_wildcard(&subjects, "analysis.deeply.nested"));
    }

    #[tokio::test]
    async fn test_send_message_tool_basic() {
        let registry = Arc::new(AgentRegistry::new());
        
        // Register an agent first
        registry
            .register_agent(
                "test_agent".to_string(),
                vec!["test.request".to_string()],
                None,
            )
            .await
            .unwrap();

        let tool = SendMessageTool::new(registry);

        let input = json!({
            "agent_id": "test_agent",
            "message": "Hello agent",
            "subject": "test.request"
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let output: SendMessageOutput = serde_json::from_value(result.unwrap()).unwrap();
        assert_eq!(output.agent_id, "test_agent");
        assert_eq!(output.status, "sent");
    }

    #[tokio::test]
    async fn test_send_message_agent_not_found() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = SendMessageTool::new(registry);

        let input = json!({
            "agent_id": "nonexistent",
            "message": "Hello"
        });

        let result = tool.execute(input).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_send_message_empty_message() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = SendMessageTool::new(registry);

        let input = json!({
            "agent_id": "agent1",
            "message": ""
        });

        let result = tool.execute(input).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_tool_metadata() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = SendMessageTool::new(registry);

        assert_eq!(tool.name(), "send_message");
        assert!(tool.description().contains("Send a message"));
        
        let schema = tool.input_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"].get("agent_id").is_some());
        assert!(schema["properties"].get("message").is_some());
    }
}
