use std::time::Duration;
use serde::{Deserialize, Serialize};

/// Configuration for LMStudio client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LmStudioConfig {
    /// Base URL of LMStudio API (e.g., "http://192.168.1.7:1234")
    pub base_url: String,

    /// Model name to use (e.g., "default", "mistral", etc.)
    pub model: String,

    /// Temperature for sampling (0.0 to 2.0). Higher = more creative.
    pub temperature: f32,

    /// Maximum tokens to generate in responses.
    pub max_tokens: u32,

    /// Request timeout duration.
    #[serde(skip)]
    pub timeout: Duration,

    /// Optional API token for authentication (Bearer token).
    #[serde(skip)]
    pub api_token: Option<String>,
}

impl Default for LmStudioConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:1234".to_string(),
            model: "default".to_string(),
            temperature: 0.7,
            max_tokens: 1000,
            timeout: Duration::from_secs(30),
            api_token: None,
        }
    }
}

impl LmStudioConfig {
    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            model: model.into(),
            ..Default::default()
        }
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_api_token(mut self, token: impl Into<String>) -> Self {
        self.api_token = Some(token.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = LmStudioConfig::default();
        assert_eq!(config.base_url, "http://localhost:1234");
        assert_eq!(config.model, "default");
        assert_eq!(config.temperature, 0.7);
        assert_eq!(config.max_tokens, 1000);
    }

    #[test]
    fn test_config_with_api_token() {
        let config = LmStudioConfig::new("http://example.com:1234", "mistral")
            .with_api_token("sk-test-token");

        assert_eq!(config.api_token, Some("sk-test-token".to_string()));
    }
}
