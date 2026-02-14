//! MCP Tools for agent system interaction
//!
//! The MCP server exposes 5 main tools for Claude:
//! 1. spawn_agent - Create new agents with system prompts and subjects
//! 2. send_message - Send messages to specific agents
//! 3. broadcast_message - Broadcast messages to agents matching filters
//! 4. list_agents - Query available agents
//! 5. query_agent_state - Get detailed agent information

pub mod agent_spawn;
pub mod broadcast_message;
pub mod list_agents;
pub mod query_agent;
pub mod send_message;

pub use agent_spawn::AgentSpawnTool;
pub use broadcast_message::AgentBroadcastTool;
pub use list_agents::AgentListTool;
pub use query_agent::AgentQueryTool;
pub use send_message::SendMessageTool;

use serde::{Deserialize, Serialize};

/// Represents a tool parameter for MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub required: bool,
}

/// Base trait for MCP tools
pub trait McpTool: Send + Sync {
    /// Tool name (exposed to Claude)
    fn name(&self) -> &str;

    /// Tool description
    fn description(&self) -> &str;

    /// Tool input schema
    fn input_schema(&self) -> serde_json::Value;

    /// Execute the tool with given input
    fn execute(&self, input: serde_json::Value) -> impl std::future::Future<Output = anyhow::Result<serde_json::Value>> + Send;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_parameter_creation() {
        let param = ToolParameter {
            name: "agent_id".to_string(),
            description: "The ID of the agent".to_string(),
            param_type: "string".to_string(),
            required: true,
        };

        assert_eq!(param.name, "agent_id");
        assert!(param.required);
    }
}
