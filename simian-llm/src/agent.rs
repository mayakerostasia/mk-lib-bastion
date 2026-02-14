//! LLM Agent implementation with subject management and system prompts
//!
//! The `LlmAgent` provides a high-level abstraction for agents that interact with LLM backends.
//! It supports:
//! - Dynamic subject subscriptions (agents choose what to listen to)
//! - System prompts (define agent role and constraints)
//! - Both completion and chat APIs
//! - Message routing based on subject filters
//!
//! # System Prompts
//!
//! System prompts are immutable and define the agent's role and constraints. They follow
//! OpenAI/Anthropic best practices:
//! - Start with role definition ("You are a...")
//! - Specify constraints and capabilities
//! - Define output format expectations
//! - Set tone/style guidance
//! - Optimal length: <1000 tokens
//!
//! For chat APIs: prepended as native system message
//! For completion APIs: prepended with "### SYSTEM INSTRUCTIONS" delimiter

use crate::client::LlmClient;
use crate::types::ChatMessage;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// An LLM-powered agent with subject subscriptions and system prompts
///
/// # Design Philosophy
///
/// - **Immutable system prompt**: Define agent role once via `Arc<String>`
/// - **Mutable subjects**: Agents can dynamically add/remove subjects at runtime
/// - **Subject-based routing**: Messages filtered by subscribed subjects
/// - **Client agnostic**: Works with any LlmClient implementation
#[derive(Clone)]
pub struct LlmAgent<L: LlmClient> {
    /// Unique agent identifier
    pub id: String,
    /// LLM client for inference
    pub client: L,
    /// System prompt defining agent role (immutable, cheap to clone)
    system_prompt: Arc<Option<String>>,
    /// Subjects this agent listens to (mutable via async methods)
    subjects: Arc<RwLock<Vec<String>>>,
}

impl<L: LlmClient> LlmAgent<L> {
    /// Create a new agent with the given ID and client
    pub fn new(id: String, client: L) -> Self {
        Self {
            id,
            client,
            system_prompt: Arc::new(None),
            subjects: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create a new agent with a system prompt
    pub fn with_system_prompt(mut self, prompt: String) -> Self {
        self.system_prompt = Arc::new(Some(prompt));
        self
    }

    /// Create a new agent with initial subjects
    pub fn with_subjects(mut self, subjects: Vec<String>) -> Self {
        // We need to reconstruct since subjects is in a RwLock
        let subjects_arc = Arc::new(RwLock::new(subjects));
        self.subjects = subjects_arc;
        self
    }

    // ========== Subject Management ==========

    /// Subscribe to a subject
    pub async fn subscribe_to_subject(&self, subject: String) -> Result<(), String> {
        let mut subjects = self.subjects.write().await;

        if subjects.contains(&subject) {
            return Err(format!("Already subscribed to '{}'", subject));
        }

        subjects.push(subject);
        Ok(())
    }

    /// Unsubscribe from a subject
    pub async fn unsubscribe_from_subject(&self, subject: &str) -> Result<(), String> {
        let mut subjects = self.subjects.write().await;

        if let Some(pos) = subjects.iter().position(|s| s == subject) {
            subjects.remove(pos);
            Ok(())
        } else {
            Err(format!("Not subscribed to '{}'", subject))
        }
    }

    /// Get all subscribed subjects (returns clone for consistency)
    pub async fn get_subjects(&self) -> Vec<String> {
        self.subjects.read().await.clone()
    }

    /// Check if agent listens to a subject (non-blocking, supports wildcards)
    ///
    /// Uses try_read() to avoid blocking in critical paths.
    /// Supports exact match and wildcard patterns:
    /// - Exact: "analysis.request" matches subscription "analysis.request"
    /// - Wildcard: "data.import" matches subscription "data.*"
    pub fn listens_to_subject(&self, subject: &str) -> bool {
        match self.subjects.try_read() {
            Ok(subs) => self.subject_matches(&subs, subject),
            Err(_) => {
                // Lock contention - return false to avoid blocking
                debug!("Subject check contention for {}", subject);
                false
            }
        }
    }

    /// Check if a subject matches any subscribed subjects (with wildcard support)
    fn subject_matches(&self, subscriptions: &[String], subject: &str) -> bool {
        subscriptions.iter().any(|sub| {
            // Exact match
            if sub == subject {
                return true;
            }

            // Wildcard match: "a.*" matches "a.b" but NOT "a.b.c"
            if let Some(prefix) = sub.strip_suffix(".*") {
                if subject.starts_with(prefix) && subject[prefix.len()..].starts_with('.') {
                    let after_dot = &subject[prefix.len() + 1..];
                    return !after_dot.contains('.');
                }
            }

            false
        })
    }

    // ========== Message Handling ==========

    /// Handle a message (check subject filter)
    pub async fn handle_message(&self, subject: &str, _content: &str) -> bool {
        if !self.listens_to_subject(subject) {
            debug!(
                agent_id = %self.id,
                subject = %subject,
                "Message filtered - not subscribed to subject"
            );
            return false;
        }

        info!(
            agent_id = %self.id,
            subject = %subject,
            "Processing message"
        );

        true
    }

    /// Send a completion request (using system prompt if available)
    pub async fn send_completion_request(&self, prompt: &str) -> Result<String> {
        let formatted_prompt = if let Some(sys_prompt) = self.system_prompt.as_ref() {
            format!("### SYSTEM INSTRUCTIONS\n{}\n\n### USER REQUEST\n{}", sys_prompt, prompt)
        } else {
            prompt.to_string()
        };

        let response = self.client.complete(&formatted_prompt).await?;
        Ok(response.content)
    }

    /// Send a chat request (using system prompt if available)
    pub async fn send_chat_request(&self, mut messages: Vec<ChatMessage>) -> Result<String> {
        // Prepend system message if system prompt is set
        if let Some(sys_prompt) = self.system_prompt.as_ref() {
            messages.insert(0, ChatMessage::system(sys_prompt.clone()));
        }

        let response = self.client.chat(&messages).await?;
        Ok(response.content)
    }

    /// Get the system prompt (if set)
    pub fn get_system_prompt(&self) -> Option<String> {
        self.system_prompt.as_ref().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::LlmClient;
    use crate::protocol::{LlmChatResponse, LlmReasonResponse};
    use crate::types::{ChatMessage, TokenUsage};
    use async_trait::async_trait;

    /// Mock LLM client for testing
    #[derive(Clone, Debug)]
    struct MockLlmClient {
        model: crate::types::ModelInfo,
    }

    impl Default for MockLlmClient {
        fn default() -> Self {
            Self {
                model: crate::types::ModelInfo {
                    name: "mock".to_string(),
                    provider: "mock".to_string(),
                    max_tokens: Some(2048),
                },
            }
        }
    }

    #[async_trait::async_trait]
    impl LlmClient for MockLlmClient {
        async fn complete(&self, _prompt: &str) -> Result<crate::types::LlmResponse> {
            Ok(crate::types::LlmResponse {
                content: "Mock response".to_string(),
                model: "mock".to_string(),
                usage: None,
            })
        }

        async fn chat(&self, _messages: &[ChatMessage]) -> Result<crate::types::LlmResponse> {
            Ok(crate::types::LlmResponse {
                content: "Mock response".to_string(),
                model: "mock".to_string(),
                usage: None,
            })
        }

        async fn health_check(&self) -> Result<bool> {
            Ok(true)
        }

        fn model_info(&self) -> &crate::types::ModelInfo {
            &self.model
        }
    }

    #[tokio::test]
    async fn test_new_agent() {
        let agent = LlmAgent::new("test-agent".to_string(), MockLlmClient::default());

        assert_eq!(agent.id, "test-agent");
        assert!(agent.get_system_prompt().is_none());
        assert!(agent.get_subjects().await.is_empty());
    }

    #[tokio::test]
    async fn test_with_system_prompt() {
        let agent = LlmAgent::new("test-agent".to_string(), MockLlmClient::default())
            .with_system_prompt("You are helpful".to_string());

        assert_eq!(
            agent.get_system_prompt(),
            Some("You are helpful".to_string())
        );
    }

    #[tokio::test]
    async fn test_with_subjects() {
        let agent = LlmAgent::new("test-agent".to_string(), MockLlmClient::default())
            .with_subjects(vec!["analysis.request".to_string()]);

        let subjects = agent.get_subjects().await;
        assert_eq!(subjects, vec!["analysis.request".to_string()]);
    }

    #[tokio::test]
    async fn test_subscribe_to_subject() {
        let agent = LlmAgent::new("test-agent".to_string(), MockLlmClient::default());

        let result = agent.subscribe_to_subject("test.subject".to_string()).await;

        assert!(result.is_ok());
        let subjects = agent.get_subjects().await;
        assert_eq!(subjects, vec!["test.subject".to_string()]);
    }

    #[tokio::test]
    async fn test_duplicate_subscription() {
        let agent = LlmAgent::new("test-agent".to_string(), MockLlmClient::default());

        agent
            .subscribe_to_subject("test.subject".to_string())
            .await
            .unwrap();

        let result = agent.subscribe_to_subject("test.subject".to_string()).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_unsubscribe() {
        let agent = LlmAgent::new("test-agent".to_string(), MockLlmClient::default())
            .with_subjects(vec!["test.subject".to_string()]);

        let result = agent.unsubscribe_from_subject("test.subject").await;

        assert!(result.is_ok());
        assert!(agent.get_subjects().await.is_empty());
    }

    #[tokio::test]
    async fn test_unsubscribe_nonexistent() {
        let agent = LlmAgent::new("test-agent".to_string(), MockLlmClient::default());

        let result = agent.unsubscribe_from_subject("nonexistent").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_listens_to_exact_subject() {
        let agent = LlmAgent::new("test-agent".to_string(), MockLlmClient::default())
            .with_subjects(vec!["analysis.request".to_string()]);

        assert!(agent.listens_to_subject("analysis.request"));
        assert!(!agent.listens_to_subject("analysis.response"));
    }

    #[tokio::test]
    async fn test_listens_to_wildcard() {
        let agent = LlmAgent::new("test-agent".to_string(), MockLlmClient::default())
            .with_subjects(vec!["data.*".to_string()]);

        assert!(agent.listens_to_subject("data.import"));
        assert!(agent.listens_to_subject("data.export"));
        assert!(!agent.listens_to_subject("other.import"));
    }

    #[tokio::test]
    async fn test_wildcard_no_recursive() {
        let agent = LlmAgent::new("test-agent".to_string(), MockLlmClient::default())
            .with_subjects(vec!["a.*".to_string()]);

        assert!(agent.listens_to_subject("a.b"));
        assert!(!agent.listens_to_subject("a.b.c"));
    }

    #[tokio::test]
    async fn test_handle_message_subscribed() {
        let agent = LlmAgent::new("test-agent".to_string(), MockLlmClient::default())
            .with_subjects(vec!["test.subject".to_string()]);

        let result = agent.handle_message("test.subject", "test message").await;

        assert!(result);
    }

    #[tokio::test]
    async fn test_handle_message_not_subscribed() {
        let agent = LlmAgent::new("test-agent".to_string(), MockLlmClient::default())
            .with_subjects(vec!["test.subject".to_string()]);

        let result = agent.handle_message("other.subject", "test message").await;

        assert!(!result);
    }

    #[tokio::test]
    async fn test_send_completion_request_no_prompt() {
        let agent = LlmAgent::new("test-agent".to_string(), MockLlmClient::default());

        let result = agent.send_completion_request("Test prompt").await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Mock response");
    }

    #[tokio::test]
    async fn test_send_completion_request_with_prompt() {
        let agent = LlmAgent::new("test-agent".to_string(), MockLlmClient::default())
            .with_system_prompt("You are helpful".to_string());

        let result = agent.send_completion_request("Test prompt").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_chat_request_no_system_prompt() {
        let agent = LlmAgent::new("test-agent".to_string(), MockLlmClient::default());

        let messages = vec![ChatMessage::user("Hello".to_string())];
        let result = agent.send_chat_request(messages).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_chat_request_with_system_prompt() {
        let agent = LlmAgent::new("test-agent".to_string(), MockLlmClient::default())
            .with_system_prompt("You are helpful".to_string());

        let messages = vec![ChatMessage::user("Hello".to_string())];
        let result = agent.send_chat_request(messages).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_multiple_subjects() {
        let agent = LlmAgent::new("test-agent".to_string(), MockLlmClient::default())
            .with_subjects(vec![
                "analysis.request".to_string(),
                "analysis.query".to_string(),
            ]);

        assert!(agent.listens_to_subject("analysis.request"));
        assert!(agent.listens_to_subject("analysis.query"));
        assert!(!agent.listens_to_subject("analysis.response"));
    }
}

