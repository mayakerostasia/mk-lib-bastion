use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::{ChatMessage, TokenUsage};

/// Request payload for an LLM reasoning operation via ACP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmReasonRequest {
    pub prompt: String,
    pub context: Option<HashMap<String, String>>,
}

/// Response payload for an LLM reasoning operation via ACP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmReasonResponse {
    pub response: String,
    pub model: String,
    pub tokens_used: Option<TokenUsage>,
}

/// Request payload for an LLM chat operation via ACP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmChatRequest {
    pub messages: Vec<ChatMessage>,
    pub context: Option<HashMap<String, String>>,
}

/// Response payload for an LLM chat operation via ACP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmChatResponse {
    pub message: ChatMessage,
    pub model: String,
    pub tokens_used: Option<TokenUsage>,
}

/// Well-known ACP subject patterns for LLM operations.
pub mod subjects {
    pub const LLM_REASON_REQUEST: &str = "llm.reason.request";
    pub const LLM_REASON_RESPONSE: &str = "llm.reason.response";
    pub const LLM_CHAT_REQUEST: &str = "llm.chat.request";
    pub const LLM_CHAT_RESPONSE: &str = "llm.chat.response";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reason_request_serde() {
        let mut ctx = HashMap::new();
        ctx.insert("task".to_string(), "summarize".to_string());

        let req = LlmReasonRequest {
            prompt: "Explain Rust ownership.".to_string(),
            context: Some(ctx),
        };
        let json = serde_json::to_string(&req).unwrap();
        let parsed: LlmReasonRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.prompt, "Explain Rust ownership.");
        assert_eq!(
            parsed.context.unwrap().get("task").unwrap(),
            "summarize"
        );
    }

    #[test]
    fn test_reason_response_serde() {
        let resp = LlmReasonResponse {
            response: "Ownership ensures memory safety.".to_string(),
            model: "llama-3".to_string(),
            tokens_used: Some(TokenUsage {
                prompt_tokens: Some(10),
                completion_tokens: Some(8),
                total_tokens: Some(18),
            }),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: LlmReasonResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.response, "Ownership ensures memory safety.");
        assert_eq!(parsed.model, "llama-3");
    }

    #[test]
    fn test_chat_request_serde() {
        let req = LlmChatRequest {
            messages: vec![
                ChatMessage::system("You are a helpful assistant."),
                ChatMessage::user("Hello!"),
            ],
            context: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        let parsed: LlmChatRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.messages.len(), 2);
        assert!(parsed.context.is_none());
    }

    #[test]
    fn test_chat_response_serde() {
        let resp = LlmChatResponse {
            message: ChatMessage::assistant("Hi there!"),
            model: "test-model".to_string(),
            tokens_used: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: LlmChatResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.message.content, "Hi there!");
        assert!(parsed.tokens_used.is_none());
    }

    #[test]
    fn test_subject_constants() {
        assert_eq!(subjects::LLM_REASON_REQUEST, "llm.reason.request");
        assert_eq!(subjects::LLM_REASON_RESPONSE, "llm.reason.response");
        assert_eq!(subjects::LLM_CHAT_REQUEST, "llm.chat.request");
        assert_eq!(subjects::LLM_CHAT_RESPONSE, "llm.chat.response");
    }
}
