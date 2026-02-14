use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

use crate::types::{ChatMessage, TokenUsage};

/// Request payload for an LLM reasoning operation via ACP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmReasonRequest {
    pub prompt: String,
    pub context: Option<HashMap<String, String>>,
}

impl LlmReasonRequest {
    /// Parses a reason request from JSON bytes, logging any errors.
    pub fn from_bytes(data: &[u8]) -> anyhow::Result<Self> {
        match serde_json::from_slice::<Self>(data) {
            Ok(req) => {
                debug!(
                    prompt_len = req.prompt.len(),
                    has_context = req.context.is_some(),
                    "Parsed LlmReasonRequest"
                );
                Ok(req)
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to parse LlmReasonRequest");
                Err(e.into())
            }
        }
    }
}

/// Response payload for an LLM reasoning operation via ACP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmReasonResponse {
    pub response: String,
    pub model: String,
    pub tokens_used: Option<TokenUsage>,
}

impl LlmReasonResponse {
    /// Parses a reason response from JSON bytes, logging any errors.
    pub fn from_bytes(data: &[u8]) -> anyhow::Result<Self> {
        match serde_json::from_slice::<Self>(data) {
            Ok(resp) => {
                debug!(
                    response_len = resp.response.len(),
                    model = %resp.model,
                    tokens = resp.tokens_used.as_ref().and_then(|t| t.total_tokens),
                    "Parsed LlmReasonResponse"
                );
                Ok(resp)
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to parse LlmReasonResponse");
                Err(e.into())
            }
        }
    }
}

/// Request payload for an LLM chat operation via ACP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmChatRequest {
    pub messages: Vec<ChatMessage>,
    pub context: Option<HashMap<String, String>>,
}

impl LlmChatRequest {
    /// Parses a chat request from JSON bytes, logging any errors.
    pub fn from_bytes(data: &[u8]) -> anyhow::Result<Self> {
        match serde_json::from_slice::<Self>(data) {
            Ok(req) => {
                debug!(
                    message_count = req.messages.len(),
                    has_context = req.context.is_some(),
                    "Parsed LlmChatRequest"
                );
                Ok(req)
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to parse LlmChatRequest");
                Err(e.into())
            }
        }
    }
}

/// Response payload for an LLM chat operation via ACP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmChatResponse {
    pub message: ChatMessage,
    pub model: String,
    pub tokens_used: Option<TokenUsage>,
}

impl LlmChatResponse {
    /// Parses a chat response from JSON bytes, logging any errors.
    pub fn from_bytes(data: &[u8]) -> anyhow::Result<Self> {
        match serde_json::from_slice::<Self>(data) {
            Ok(resp) => {
                debug!(
                    response_len = resp.message.content.len(),
                    model = %resp.model,
                    tokens = resp.tokens_used.as_ref().and_then(|t| t.total_tokens),
                    "Parsed LlmChatResponse"
                );
                Ok(resp)
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to parse LlmChatResponse");
                Err(e.into())
            }
        }
    }
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
    fn test_reason_request_from_bytes() {
        let req = LlmReasonRequest {
            prompt: "Test".to_string(),
            context: None,
        };
        let bytes = serde_json::to_vec(&req).unwrap();
        let parsed = LlmReasonRequest::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.prompt, "Test");
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
    fn test_reason_response_from_bytes() {
        let resp = LlmReasonResponse {
            response: "Test response".to_string(),
            model: "test".to_string(),
            tokens_used: None,
        };
        let bytes = serde_json::to_vec(&resp).unwrap();
        let parsed = LlmReasonResponse::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.response, "Test response");
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
    fn test_chat_request_from_bytes() {
        let req = LlmChatRequest {
            messages: vec![ChatMessage::user("Hi")],
            context: None,
        };
        let bytes = serde_json::to_vec(&req).unwrap();
        let parsed = LlmChatRequest::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.messages.len(), 1);
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
    fn test_chat_response_from_bytes() {
        let resp = LlmChatResponse {
            message: ChatMessage::assistant("Test"),
            model: "test".to_string(),
            tokens_used: None,
        };
        let bytes = serde_json::to_vec(&resp).unwrap();
        let parsed = LlmChatResponse::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.message.content, "Test");
    }

    #[test]
    fn test_subject_constants() {
        assert_eq!(subjects::LLM_REASON_REQUEST, "llm.reason.request");
        assert_eq!(subjects::LLM_REASON_RESPONSE, "llm.reason.response");
        assert_eq!(subjects::LLM_CHAT_REQUEST, "llm.chat.request");
        assert_eq!(subjects::LLM_CHAT_RESPONSE, "llm.chat.response");
    }
}
