// use anyhow::Error;

#[derive(thiserror::Error, Debug)]
pub enum NSLibError {
    #[error("Boxed Error {0:#?}")]
    Boxed(Box<dyn std::error::Error + Send + Sync>),
    #[error("Nats Error: {0:#?}")]
    NatsError(#[from] async_nats::Error),
}
