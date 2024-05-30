// use anyhow::Error;

#[derive(thiserror::Error, Debug)]
pub enum NSLibError {
    #[error("Anyhow!: {0:#?}")]
    Anyhow(#[source] anyhow::Error),
    #[error("Nats Error: {0:#?}")]
    NatsError(#[from] async_nats::Error),
    #[error("Publish Error: {0:#?}")]
    PublishError(#[from] async_nats::error::Error<async_nats::client::PublishErrorKind>),
    #[error("Request Error: {0:#?}")]
    RequestError(#[from] async_nats::RequestError),
    #[error("Boxed Error {0:#?}")]
    Boxed(Box<dyn std::error::Error + Send + Sync>),
    // #[error("")]
}

// #[derive(thiserror::Error, Debug)]
// pub enum ServiceError {
//     #[error("Invalid Request: {0}")]
//     InvalidRequest(Frame)
// }
