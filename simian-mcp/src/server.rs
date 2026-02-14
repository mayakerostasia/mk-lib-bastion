//! MCP Server for Claude integration with the agent system
//!
//! The MCP server provides a bridge between Claude and the agent system,
//! allowing Claude to spawn agents, send messages, and query agent state.
//! The server is stateless - all state is managed by the agent registry.

use anyhow::Result;
use simian_llm::AgentRegistry;
use std::sync::Arc;

/// Configuration for MCP server
#[derive(Clone, Debug)]
pub struct McpServerConfig {
    /// Port to listen on for MCP connections
    pub port: u16,
    /// Base subject for NATS transport (e.g., "agents")
    pub base_subject: String,
    /// Enable verbose logging
    pub verbose: bool,
}

impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            port: 3000,
            base_subject: "agents".to_string(),
            verbose: false,
        }
    }
}

/// MCP Server instance
pub struct McpServer {
    config: McpServerConfig,
    registry: Arc<AgentRegistry>,
}

impl McpServer {
    /// Create a new MCP server
    pub fn new(config: McpServerConfig, registry: Arc<AgentRegistry>) -> Self {
        Self { config, registry }
    }

    /// Start the MCP server
    pub async fn start(&self) -> Result<()> {
        if self.config.verbose {
            tracing::info!(
                "Starting MCP server on port {} with base subject: {}",
                self.config.port,
                self.config.base_subject
            );
        }

        // Placeholder for actual MCP server implementation
        // In a real scenario, this would:
        // 1. Set up stdio/HTTP transport for MCP protocol
        // 2. Register tool handlers
        // 3. Register resource handlers
        // 4. Listen for requests from Claude

        Ok(())
    }

    /// Get reference to the agent registry
    pub fn registry(&self) -> Arc<AgentRegistry> {
        Arc::clone(&self.registry)
    }

    /// Get server configuration
    pub fn config(&self) -> &McpServerConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = McpServerConfig::default();
        assert_eq!(config.port, 3000);
        assert_eq!(config.base_subject, "agents");
        assert!(!config.verbose);
    }

    #[tokio::test]
    async fn test_mcp_server_creation() {
        let config = McpServerConfig::default();
        let registry = Arc::new(AgentRegistry::new());
        let server = McpServer::new(config, registry);

        assert_eq!(server.config().port, 3000);
    }

    #[tokio::test]
    async fn test_mcp_server_start() {
        let config = McpServerConfig::default();
        let registry = Arc::new(AgentRegistry::new());
        let server = McpServer::new(config, registry);

        let result = server.start().await;
        assert!(result.is_ok());
    }
}
