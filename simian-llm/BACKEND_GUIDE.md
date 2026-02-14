# simian-llm Backend Implementation Guide

A guide to implementing new LLM backend clients for simian-llm.

## Table of Contents

1. [Overview](#overview)
2. [Implementing LlmClient](#implementing-llmclient)
3. [Example: OpenAI Client](#example-openai-client)
4. [Best Practices](#best-practices)
5. [Testing](#testing)

## Overview

Any LLM API can be integrated by implementing the `LlmClient` trait. This guide walks through the process.

### What You Need

1. **API Endpoint** - The LLM service's HTTP/gRPC endpoint
2. **Request Format** - How to format requests (REST body, parameters, etc.)
3. **Response Format** - How to parse responses
4. **Authentication** - How to authenticate (API key, OAuth, etc.)
5. **Error Handling** - How the API signals errors

## Implementing LlmClient

### Step 1: Define Your Client Struct

```rust
use simian_llm::LlmClient;
use reqwest::Client;
use std::time::Duration;

/// OpenAI LLM client
#[derive(Clone, Debug)]
pub struct OpenAiClient {
    http_client: Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl OpenAiClient {
    pub async fn new(api_key: String, model: String) -> anyhow::Result<Self> {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()?;

        Ok(Self {
            http_client,
            api_key,
            model,
            base_url: "https://api.openai.com/v1".to_string(),
        })
    }
}
```

### Step 2: Define Request/Response Types

Use `serde` to define structures that match your API:

```rust
use serde::{Deserialize, Serialize};

/// OpenAI completion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiCompletionRequest {
    pub model: String,
    pub prompt: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

/// OpenAI completion response
#[derive(Debug, Clone, Deserialize)]
pub struct OpenAiCompletionResponse {
    pub choices: Vec<OpenAiChoice>,
    pub usage: OpenAiUsage,
    pub model: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenAiChoice {
    pub text: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenAiUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}
```

### Step 3: Implement the Trait

```rust
use async_trait::async_trait;
use simian_llm::{LlmClient, ChatMessage, LlmResponse, ModelInfo, ChatRole};
use anyhow::Result;

#[async_trait]
impl LlmClient for OpenAiClient {
    async fn complete(&self, prompt: &str) -> Result<LlmResponse> {
        // Build request
        let request = OpenAiCompletionRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            temperature: 0.7,
            max_tokens: 1000,
        };

        // Make HTTP call
        let response = self
            .http_client
            .post(format!("{}/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await?;

        // Parse response
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!(
                "OpenAI API error ({}): {}",
                status,
                error_text
            ));
        }

        let api_response: OpenAiCompletionResponse = response.json().await?;

        // Convert to SDK types
        let content = api_response
            .choices
            .first()
            .map(|c| c.text.clone())
            .unwrap_or_default();

        Ok(LlmResponse {
            content,
            model: api_response.model,
            usage: Some(TokenUsage {
                prompt_tokens: Some(api_response.usage.prompt_tokens),
                completion_tokens: Some(api_response.usage.completion_tokens),
                total_tokens: Some(api_response.usage.total_tokens),
            }),
        })
    }

    async fn chat(&self, messages: &[ChatMessage]) -> Result<LlmResponse> {
        // Convert ChatMessage to OpenAI format
        let openai_messages: Vec<OpenAiMessage> = messages
            .iter()
            .map(|m| OpenAiMessage {
                role: match m.role {
                    ChatRole::System => "system",
                    ChatRole::User => "user",
                    ChatRole::Assistant => "assistant",
                },
                content: m.content.clone(),
            })
            .collect();

        // Build request
        let request = OpenAiChatRequest {
            model: self.model.clone(),
            messages: openai_messages,
            temperature: 0.7,
            max_tokens: 1000,
        };

        // Make HTTP call
        let response = self
            .http_client
            .post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await?;

        // Parse and convert
        let api_response: OpenAiChatResponse = response.json().await?;

        let content = api_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(LlmResponse {
            content,
            model: api_response.model,
            usage: Some(TokenUsage {
                prompt_tokens: Some(api_response.usage.prompt_tokens),
                completion_tokens: Some(api_response.usage.completion_tokens),
                total_tokens: Some(api_response.usage.total_tokens),
            }),
        })
    }

    async fn health_check(&self) -> Result<bool> {
        match self.http_client
            .get(format!("{}/models", self.base_url))
            .bearer_auth(&self.api_key)
            .send()
            .await
        {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    fn model_info(&self) -> &ModelInfo {
        // Store ModelInfo in the struct for references
        &self.model_info  // You'll need to add this field
    }
}
```

## Example: OpenAI Client

Here's a complete, production-ready OpenAI client implementation:

```rust
// openai_client.rs

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use simian_llm::{ChatMessage, ChatRole, LlmClient, LlmResponse, ModelInfo, TokenUsage};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info};

#[derive(Clone, Debug)]
pub struct OpenAiClient {
    http_client: Client,
    api_key: String,
    model: String,
    base_url: String,
    model_info: Arc<ModelInfo>,
}

impl OpenAiClient {
    pub async fn new(api_key: String, model: String) -> anyhow::Result<Self> {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()?;

        let model_info = Arc::new(ModelInfo {
            name: model.clone(),
            provider: "openai".to_string(),
            max_tokens: Some(128000),
        });

        info!(model = %model, "OpenAI client initialized");

        Ok(Self {
            http_client,
            api_key,
            model,
            base_url: "https://api.openai.com/v1".to_string(),
            model_info,
        })
    }

    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }
}

#[derive(Debug, Clone, Serialize)]
struct OpenAiCompletionRequest {
    model: String,
    prompt: String,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Debug, Clone, Deserialize)]
struct OpenAiCompletionResponse {
    choices: Vec<OpenAiChoice>,
    usage: OpenAiUsage,
    model: String,
}

#[derive(Debug, Clone, Deserialize)]
struct OpenAiChoice {
    text: String,
}

#[derive(Debug, Clone, Serialize)]
struct OpenAiChatRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(Debug, Clone, Deserialize)]
struct OpenAiChatResponse {
    choices: Vec<OpenAiChatChoice>,
    usage: OpenAiUsage,
    model: String,
}

#[derive(Debug, Clone, Deserialize)]
struct OpenAiChatChoice {
    message: OpenAiMessage,
}

#[derive(Debug, Clone, Deserialize)]
struct OpenAiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[async_trait]
impl LlmClient for OpenAiClient {
    async fn complete(&self, prompt: &str) -> anyhow::Result<LlmResponse> {
        metrics::counter!("llm.complete.requests").increment(1);

        let request = OpenAiCompletionRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            temperature: 0.7,
            max_tokens: 1000,
        };

        info!(prompt_len = prompt.len(), "Calling OpenAI completions");

        let response = self
            .http_client
            .post(format!("{}/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                metrics::counter!("llm.complete.errors").increment(1);
                error!(error = %e, "OpenAI request failed");
                anyhow::anyhow!("OpenAI API request failed: {}", e)
            })?;

        let status = response.status();
        let api_response: OpenAiCompletionResponse = response.json().await.map_err(|e| {
            metrics::counter!("llm.complete.errors").increment(1);
            error!(error = %e, status = %status, "Failed to parse OpenAI response");
            anyhow::anyhow!("Failed to parse response: {}", e)
        })?;

        let content = api_response
            .choices
            .first()
            .map(|c| c.text.clone())
            .unwrap_or_default();

        info!(
            response_len = content.len(),
            tokens = api_response.usage.total_tokens,
            "OpenAI completion succeeded"
        );

        Ok(LlmResponse {
            content,
            model: api_response.model,
            usage: Some(TokenUsage {
                prompt_tokens: Some(api_response.usage.prompt_tokens),
                completion_tokens: Some(api_response.usage.completion_tokens),
                total_tokens: Some(api_response.usage.total_tokens),
            }),
        })
    }

    async fn chat(&self, messages: &[ChatMessage]) -> anyhow::Result<LlmResponse> {
        metrics::counter!("llm.chat.requests").increment(1);

        let openai_messages = messages
            .iter()
            .map(|m| OpenAiMessage {
                role: match m.role {
                    ChatRole::System => "system",
                    ChatRole::User => "user",
                    ChatRole::Assistant => "assistant",
                },
                content: m.content.clone(),
            })
            .collect();

        let request = OpenAiChatRequest {
            model: self.model.clone(),
            messages: openai_messages,
            temperature: 0.7,
            max_tokens: 1000,
        };

        info!(message_count = messages.len(), "Calling OpenAI chat");

        let response = self
            .http_client
            .post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                metrics::counter!("llm.chat.errors").increment(1);
                error!(error = %e, "OpenAI chat request failed");
                anyhow::anyhow!("OpenAI chat request failed: {}", e)
            })?;

        let api_response: OpenAiChatResponse = response.json().await.map_err(|e| {
            metrics::counter!("llm.chat.errors").increment(1);
            error!(error = %e, "Failed to parse OpenAI chat response");
            anyhow::anyhow!("Failed to parse chat response: {}", e)
        })?;

        let content = api_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        info!(
            response_len = content.len(),
            tokens = api_response.usage.total_tokens,
            "OpenAI chat succeeded"
        );

        Ok(LlmResponse {
            content,
            model: api_response.model,
            usage: Some(TokenUsage {
                prompt_tokens: Some(api_response.usage.prompt_tokens),
                completion_tokens: Some(api_response.usage.completion_tokens),
                total_tokens: Some(api_response.usage.total_tokens),
            }),
        })
    }

    async fn health_check(&self) -> anyhow::Result<bool> {
        metrics::counter!("llm.health_checks").increment(1);

        match self
            .http_client
            .get(format!("{}/models", self.base_url))
            .bearer_auth(&self.api_key)
            .send()
            .await
        {
            Ok(resp) => {
                if resp.status().is_success() {
                    info!("OpenAI health check passed");
                    Ok(true)
                } else {
                    error!(status = %resp.status(), "OpenAI health check failed");
                    Ok(false)
                }
            }
            Err(e) => {
                error!(error = %e, "OpenAI health check error");
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
    fn test_client_creation() {
        let _client = tokio::runtime::Runtime::new().unwrap().block_on(async {
            OpenAiClient::new("test-key".to_string(), "gpt-4".to_string()).await
        });
    }
}
```

## Best Practices

### 1. Configuration with Builder Pattern

```rust
pub struct LlmConfig {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
    pub timeout: Duration,
    pub temperature: f32,
    pub max_tokens: u32,
}

impl LlmConfig {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            base_url: "https://api.example.com/v1".to_string(),
            timeout: Duration::from_secs(60),
            temperature: 0.7,
            max_tokens: 1000,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }

    // ... more builder methods
}
```

### 2. Error Handling

Always map API errors to meaningful anyhow::Result:

```rust
let response = self.http_client.post(url).send().await.map_err(|e| {
    error!(error = %e, "HTTP request failed");
    anyhow::anyhow!("API request failed: {}", e)
})?;

let status = response.status();
if !status.is_success() {
    let error_text = response.text().await.unwrap_or_default();
    error!(status = %status, error = %error_text, "API error");
    return Err(anyhow::anyhow!("API error ({}): {}", status, error_text));
}
```

### 3. Instrumentation

Include tracing and metrics:

```rust
// At method start
info!(model = %self.model, prompt_len = prompt.len(), "Starting LLM request");
metrics::counter!("llm.requests").increment(1);

// On error
error!(error = %e, "LLM request failed");
metrics::counter!("llm.errors").increment(1);

// On success
info!(tokens = usage.total_tokens, "LLM request succeeded");
```

### 4. Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct MockHttpClient;

    #[test]
    fn test_message_conversion() {
        let msg = ChatMessage::user("Hello");
        assert_eq!(msg.content, "Hello");
    }

    #[tokio::test]
    #[ignore] // Only run with real API key
    async fn test_real_api() {
        let client = OpenAiClient::new(
            std::env::var("OPENAI_API_KEY").unwrap(),
            "gpt-3.5-turbo".to_string(),
        )
        .await
        .unwrap();

        let response = client
            .complete("Say hello")
            .await
            .unwrap();

        assert!(!response.content.is_empty());
    }
}
```

---

**Next:** See [EXAMPLES.md](./EXAMPLES.md) for common usage patterns.
