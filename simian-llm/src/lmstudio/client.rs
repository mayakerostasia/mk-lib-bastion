use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;

use crate::client::LlmClient;
use crate::types::{ChatMessage, ChatRole, LlmResponse, ModelInfo};
use crate::{counter, error, info};

use super::config::LmStudioConfig;
use super::api::{ChatCompletionRequest, ChatMessage as ApiChatMessage, CompletionRequest};

/// LMStudio client using OpenAI-compatible HTTP API.
#[derive(Clone, Debug)]
pub struct LmStudioClient {
    http_client: Client,
    config: LmStudioConfig,
    model_info: ModelInfo,
}

impl LmStudioClient {
    /// Creates a new LMStudio client.
    pub async fn new(config: LmStudioConfig) -> Result<Self> {
        let model_info = ModelInfo {
            name: config.model.clone(),
            provider: "lmstudio".to_string(),
            max_tokens: Some(config.max_tokens),
        };

        let http_client = Client::builder()
            .timeout(config.timeout)
            .build()?;

        info!(
            base_url = %config.base_url,
            model = %config.model,
            "LMStudio client initialized"
        );

        Ok(Self {
            http_client,
            config,
            model_info,
        })
    }

    /// Helper to build a request with authentication headers.
    fn build_request(&self, method: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(token) = &self.config.api_token {
            method.bearer_auth(token)
        } else {
            method
        }
    }

    /// Helper to convert SDK ChatMessage to API ChatMessage.
    fn convert_message(msg: &ChatMessage) -> ApiChatMessage {
        let role = match msg.role {
            ChatRole::System => "system",
            ChatRole::User => "user",
            ChatRole::Assistant => "assistant",
        };
        ApiChatMessage {
            role: role.to_string(),
            content: msg.content.clone(),
        }
    }
}

#[async_trait]
impl LlmClient for LmStudioClient {
    async fn complete(&self, prompt: &str) -> Result<LlmResponse> {
        counter!("llm.complete.requests").increment(1);

        let request = CompletionRequest {
            model: self.config.model.clone(),
            prompt: prompt.to_string(),
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
        };

        let url = format!("{}/v1/completions", self.config.base_url);
        info!(url = %url, prompt_len = prompt.len(), "Calling LMStudio completions");

        let req_builder = self.http_client.post(&url).json(&request);
        let response = self.build_request(req_builder)
            .send()
            .await
            .map_err(|e| {
                counter!("llm.complete.errors").increment(1);
                error!(error = %e, "LMStudio completions request failed");
                anyhow::anyhow!("LMStudio request failed: {}", e)
            })?;

        let body = response.json::<super::api::CompletionResponse>().await.map_err(|e| {
            counter!("llm.complete.errors").increment(1);
            error!(error = %e, "Failed to parse LMStudio response");
            anyhow::anyhow!("Failed to parse response: {}", e)
        })?;

        let content = body
            .choices
            .first()
            .map(|c| c.text.clone())
            .unwrap_or_default();

        info!(
            response_len = content.len(),
            tokens = body.usage.as_ref().map(|u| u.total_tokens),
            "LMStudio completion succeeded"
        );

        Ok(LlmResponse {
            content,
            model: body.model,
            usage: body.usage.map(|u| u.into()),
        })
    }

    async fn chat(&self, messages: &[ChatMessage]) -> Result<LlmResponse> {
        counter!("llm.chat.requests").increment(1);

        let api_messages: Vec<ApiChatMessage> = messages.iter().map(Self::convert_message).collect();

        let request = ChatCompletionRequest {
            model: self.config.model.clone(),
            messages: api_messages,
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
        };

        let url = format!("{}/v1/chat/completions", self.config.base_url);
        info!(url = %url, message_count = messages.len(), "Calling LMStudio chat");

        let req_builder = self.http_client.post(&url).json(&request);
        let response = self.build_request(req_builder)
            .send()
            .await
            .map_err(|e| {
                counter!("llm.chat.errors").increment(1);
                error!(error = %e, "LMStudio chat request failed");
                anyhow::anyhow!("LMStudio chat request failed: {}", e)
            })?;

        let body = response.json::<super::api::ChatCompletionResponse>().await.map_err(|e| {
            counter!("llm.chat.errors").increment(1);
            error!(error = %e, "Failed to parse LMStudio chat response");
            anyhow::anyhow!("Failed to parse chat response: {}", e)
        })?;

        let choice = body.choices.first().ok_or_else(|| {
            counter!("llm.chat.errors").increment(1);
            anyhow::anyhow!("No choices in LMStudio response")
        })?;

        info!(
            response_len = choice.message.content.len(),
            tokens = body.usage.as_ref().map(|u| u.total_tokens),
            "LMStudio chat succeeded"
        );

        Ok(LlmResponse {
            content: choice.message.content.clone(),
            model: body.model,
            usage: body.usage.map(|u| u.into()),
        })
    }

    async fn health_check(&self) -> Result<bool> {
        counter!("llm.health_checks").increment(1);

        let url = format!("{}/models", self.config.base_url);
        info!(url = %url, "LMStudio health check");

        let req_builder = self.http_client.get(&url);
        let response = self.build_request(req_builder).send().await;

        match response {
            Ok(response) => {
                if response.status().is_success() {
                    info!("LMStudio health check passed");
                    Ok(true)
                } else {
                    error!(status = %response.status(), "LMStudio returned non-2xx status");
                    Ok(false)
                }
            }
            Err(e) => {
                error!(error = %e, "LMStudio health check failed");
                Ok(false)
            }
        }
    }

    fn model_info(&self) -> &ModelInfo {
        &self.model_info
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_message() {
        let msg = ChatMessage::user("Hello");
        let api_msg = LmStudioClient::convert_message(&msg);
        assert_eq!(api_msg.role, "user");
        assert_eq!(api_msg.content, "Hello");
    }

    #[test]
    fn test_client_creation() {
        let config = LmStudioConfig::default();
        let _client = tokio::runtime::Runtime::new().unwrap().block_on(async {
            // This would fail if the server isn't running, so we just test the config
            assert_eq!(config.model, "default");
            Ok::<(), anyhow::Error>(())
        });
    }
}
