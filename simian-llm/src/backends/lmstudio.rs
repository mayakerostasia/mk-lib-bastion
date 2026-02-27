use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error};

use crate::client::LlmClient;
use crate::types::{ChatMessage, LlmResponse, ModelInfo};

/// LMStudio OpenAI-compatible API client with API key authentication
#[derive(Clone, Debug)]
pub struct LmStudioClient {
    client: Client,
    base_url: String,
    api_key: String,
    model: String,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize)]
struct CompletionRequest {
    model: String,
    prompt: String,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessagePayload>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize, Deserialize)]
struct ChatMessagePayload {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct CompletionResponse {
    choices: Vec<CompletionChoice>,
}

#[derive(Deserialize)]
struct CompletionChoice {
    text: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessagePayload,
}

impl LmStudioClient {
    pub fn new(base_url: &str, api_key: &str, model: &str, temperature: f32, max_tokens: u32) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            model: model.to_string(),
            temperature,
            max_tokens,
        })
    }

    pub fn from_env() -> Result<Self> {
        let base_url = std::env::var("LMSTUDIO_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:1234/v1".to_string());
        let api_key = std::env::var("LMSTUDIO_API_KEY")
            .context("LMSTUDIO_API_KEY not set")?;
        let model = std::env::var("LMSTUDIO_MODEL")
            .unwrap_or_else(|_| "local-model".to_string());
        let temperature: f32 = std::env::var("LMSTUDIO_TEMPERATURE")
            .unwrap_or_else(|_| "0.7".to_string())
            .parse()
            .unwrap_or(0.7);
        let max_tokens: u32 = std::env::var("LMSTUDIO_MAX_TOKENS")
            .unwrap_or_else(|_| "2000".to_string())
            .parse()
            .unwrap_or(2000);

        Self::new(&base_url, &api_key, &model, temperature, max_tokens)
    }
}

#[async_trait]
impl LlmClient for LmStudioClient {
    async fn complete(&self, prompt: &str) -> Result<LlmResponse> {
        debug!(prompt_len = prompt.len(), model = %self.model, "Sending completion");

        let request = CompletionRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            temperature: self.temperature,
            max_tokens: self.max_tokens,
        };

        let response = self
            .client
            .post(format!("{}/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await
            .context("Failed to send completion request")?;

        if response.status() != StatusCode::OK {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!(status = %status, body = %body, "LLM request failed");
            anyhow::bail!("LLM request failed with status {}: {}", status, body);
        }

        let completion: CompletionResponse = response
            .json()
            .await
            .context("Failed to parse completion response")?;

        let text = completion
            .choices
            .first()
            .map(|c| c.text.clone())
            .unwrap_or_default();

        debug!(response_len = text.len(), "Received completion");

        Ok(LlmResponse {
            content: text,
            model: self.model.clone(),
            usage: None,
        })
    }

    async fn chat(&self, messages: &[ChatMessage]) -> Result<LlmResponse> {
        debug!(message_count = messages.len(), model = %self.model, "Sending chat");

        let chat_messages: Vec<ChatMessagePayload> = messages
            .iter()
            .map(|m| ChatMessagePayload {
                role: format!("{:?}", m.role).to_lowercase(),
                content: m.content.clone(),
            })
            .collect();

        let request = ChatRequest {
            model: self.model.clone(),
            messages: chat_messages,
            temperature: self.temperature,
            max_tokens: self.max_tokens,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await
            .context("Failed to send chat request")?;

        if response.status() != StatusCode::OK {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!(status = %status, body = %body, "Chat request failed");
            anyhow::bail!("Chat request failed with status {}: {}", status, body);
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .context("Failed to parse chat response")?;

        let text = chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        debug!(response_len = text.len(), "Received chat response");

        Ok(LlmResponse {
            content: text,
            model: self.model.clone(),
            usage: None,
        })
    }

    async fn health_check(&self) -> Result<bool> {
        let response = self
            .client
            .get(format!("{}/models", self.base_url))
            .bearer_auth(&self.api_key)
            .timeout(Duration::from_secs(5))
            .send()
            .await;

        match response {
            Ok(r) => Ok(r.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    fn model_info(&self) -> &ModelInfo {
        Box::leak(Box::new(ModelInfo {
            name: self.model.clone(),
            provider: "LMStudio".to_string(),
            max_tokens: Some(self.max_tokens),
        }))
    }
}
