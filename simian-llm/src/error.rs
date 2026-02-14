use thiserror::Error;

/// Errors that can occur during LLM operations.
#[derive(Error, Debug)]
pub enum LlmError {
    #[error("connection error: {0}")]
    ConnectionError(String),

    #[error("request error: {0}")]
    RequestError(String),

    #[error("response parse error: {0}")]
    ParseError(String),

    #[error("model not available: {0}")]
    ModelNotAvailable(String),

    #[error("timeout after {0}ms")]
    Timeout(u64),

    #[error("rate limited: retry after {retry_after_ms:?}ms")]
    RateLimited { retry_after_ms: Option<u64> },

    #[error("{0}")]
    Other(String),
}
