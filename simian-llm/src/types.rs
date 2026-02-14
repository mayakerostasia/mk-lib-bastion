use serde::{Deserialize, Serialize};

/// Role of a participant in a chat conversation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    System,
    User,
    Assistant,
}

/// A single message in a chat conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::System,
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::User,
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::Assistant,
            content: content.into(),
        }
    }
}

/// Metadata about an LLM model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub provider: String,
    pub max_tokens: Option<u32>,
}

/// Token usage information from a completion response.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
}

/// Response from an LLM completion or chat request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub content: String,
    pub model: String,
    pub usage: Option<TokenUsage>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_message_constructors() {
        let sys = ChatMessage::system("You are a helpful assistant.");
        assert_eq!(sys.role, ChatRole::System);
        assert_eq!(sys.content, "You are a helpful assistant.");

        let usr = ChatMessage::user("Hello");
        assert_eq!(usr.role, ChatRole::User);

        let asst = ChatMessage::assistant("Hi there!");
        assert_eq!(asst.role, ChatRole::Assistant);
    }

    #[test]
    fn test_chat_message_serde_roundtrip() {
        let msg = ChatMessage::user("test message");
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ChatMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.role, ChatRole::User);
        assert_eq!(parsed.content, "test message");
    }

    #[test]
    fn test_model_info_serde() {
        let info = ModelInfo {
            name: "llama-3".to_string(),
            provider: "lmstudio".to_string(),
            max_tokens: Some(4096),
        };
        let json = serde_json::to_string(&info).unwrap();
        let parsed: ModelInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "llama-3");
        assert_eq!(parsed.provider, "lmstudio");
        assert_eq!(parsed.max_tokens, Some(4096));
    }

    #[test]
    fn test_llm_response_serde() {
        let resp = LlmResponse {
            content: "Hello!".to_string(),
            model: "test-model".to_string(),
            usage: Some(TokenUsage {
                prompt_tokens: Some(10),
                completion_tokens: Some(5),
                total_tokens: Some(15),
            }),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: LlmResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.content, "Hello!");
        assert_eq!(parsed.usage.unwrap().total_tokens, Some(15));
    }
}
