use crate::{
    boxed_future_generator,
    core::{
        new_client, new_echo_responder, new_object_responder, new_service_future_responder,
        new_service_responder, new_tower_service_responder,
    },
    Frame, NSLibError,
};
use bytes::Bytes;
use petname::Generator;
use rand::thread_rng;
use std::future::Future;
use tokio_util::sync::CancellationToken;
use tower::BoxError;
use tracing::instrument;

pub type Error = crate::NSLibError;

#[derive(Debug)]
pub struct Kong {
    pub name: String,
    subject: String,
    client: async_nats::Client,
    token: CancellationToken,
    // bastion: Option<Bastion>,
}

impl Drop for Kong {
    fn drop(&mut self) {
        self.token.cancel();
        let _ = *self;
    }
}

impl Kong {
    pub async fn new(subject: &str, nats_url: &str) -> Result<Self, Error> {
        let mut rng = thread_rng();
        let name = petname::Petnames::default()
            .generate(&mut rng, 2, "-")
            .expect("Petname Failed");

        let nats_client = new_client(nats_url)
            .await
            .map_err(|e| NSLibError::NatsError(e.into()))?;
        Ok(Kong {
            name,
            subject: subject.to_string(),
            client: nats_client,
            token: CancellationToken::new(),
            // bastion: None,
        })
    }

    pub fn client(&self) -> Result<async_nats::Client, Error> {
        Ok(self.client.clone())
    }

    pub async fn listen(&self) -> Result<tokio::task::JoinHandle<Result<(), Error>>, Error> {
        new_echo_responder(&self.client, &self.subject).await
    }

    pub async fn serve(
        &self,
        object: impl Into<Bytes>,
    ) -> Result<tokio::task::JoinHandle<Result<(), BoxError>>, BoxError> {
        new_object_responder(&self.client, &self.subject, object).await
    }

    pub async fn service<T, U>(
        &self,
        func: fn() -> T,
    ) -> Result<tokio::task::JoinHandle<Result<(), BoxError>>, BoxError>
    where
        T: futures::Future<Output = U> + Send + 'static,
        U: Into<Bytes> + std::fmt::Debug + Send + 'static,
    {
        new_service_responder(
            &self.client,
            &self.name,
            &self.subject,
            boxed_future_generator(func),
            self.token.clone(),
        )
        .await
    }

    pub async fn service_future<O, T>(
        &self,
        func: fn(Frame) -> O,
        // func: fn() -> T,
    ) -> Result<tokio::task::JoinHandle<Result<(), BoxError>>, BoxError>
    where
        O: Future<Output = Result<T, BoxError>> + Send + 'static,
        T: Into<Bytes> + std::fmt::Debug + Send,
    {
        new_service_future_responder::<O, T>(
            &self.client,
            &self.name,
            &self.subject,
            func,
            self.token.clone(),
        )
        .await
    }

    pub async fn tower_service<S>(
        &self,
        service: S, // func: fn() -> T,
    ) -> Result<tokio::task::JoinHandle<Result<(), BoxError>>, BoxError>
    where
        S: tower::Service<Frame> + Send + Sync + Clone + 'static,
        S::Future: Send + Sync,
        S::Response: Into<Bytes> + Send + Sync + std::fmt::Debug,
        S::Error: Into<BoxError>,
    {
        new_tower_service_responder::<S>(
            &self.client,
            &self.name,
            &self.subject,
            service,
            self.token.clone(),
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::make_request;

    #[tokio::test]
    async fn initialize_kong() -> Result<(), BoxError> {
        let _kong = Kong::new("greet.kong", "10.2.4.106:4222").await;
        // assert!(true);
        Ok(())
    }

    #[tokio::test]
    async fn named_kong() -> Result<(), BoxError> {
        let kong = Kong::new("greet", "nats://10.2.4.106:4222").await?;
        let listener = kong.listen().await?;
        let client: async_nats::Client = new_client("nats://10.2.4.106:4222").await?;
        let _request = make_request(
            client.clone(),
            "greet.sue".to_string(),
            "Hello! My name is Sue!".to_string(),
        )
        .await?;
        listener.abort();
        Ok(())
    }
}
