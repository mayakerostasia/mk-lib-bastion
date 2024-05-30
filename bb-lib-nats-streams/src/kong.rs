use crate::boxed_future_generator;
use crate::core::{
    new_client, new_echo_responder, new_object_responder, new_tower_service_responder,
    new_service_responder, new_service_future_responder
};
use crate::Frame;
use bytes::Bytes;
use petname::Generator;
use rand::thread_rng;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use tower::BoxError;
use tracing::instrument;
use tracing::instrument::Instrumented;
use std::future::Future;

pub type Error = crate::NSLibError;

#[derive(Debug)]
pub struct Kong {
    pub name: String,
    subject: String,
    client: async_nats::Client,
    token: CancellationToken,
    // bastion: Option<Bastion>,
}

// impl Drop for Kong {
//     fn drop(&mut self) {
//         self.token.cancel();
//         let _ = *self;
//     }
// }

impl Kong {
    pub async fn new(subject: &str, nats_url: &str) -> Self {
        let mut rng = thread_rng();
        let name = petname::Petnames::default()
            .generate(&mut rng, 2, "-")
            .expect("Petname Failed");
        Kong {
            name,
            subject: subject.to_string(),
            client: new_client(nats_url).await.unwrap(),
            token: CancellationToken::new(),
            // bastion: None,
        }
    }

    pub fn client(&self) -> Result<async_nats::Client, Error> {
        Ok(self.client.clone())
    }

    pub async fn listen(&self) -> Result<tokio::task::JoinHandle<Result<(), Error>>, Error> {
        Ok(new_echo_responder(&self.client, &self.subject).await?)
    }

    pub async fn serve<T>(
        &self,
        object: impl Into<Bytes>,
    ) -> Result<tokio::task::JoinHandle<Result<(), BoxError>>, BoxError> {
        Ok::<_, BoxError>(new_object_responder::<T>(&self.client, &self.subject, object).await?)
    }

    #[instrument(skip(self, func), fields( kong_name = %self.name, kong_subject = %self.subject))]
    pub async fn service<T, U>(
        &self,
        func: fn() -> T,
    ) -> Result<Instrumented<tokio::task::JoinHandle<Result<(), BoxError>>>, BoxError>
    where
        T: Send + futures::Future<Output = U> + 'static,
        U: Send + 'static + Into<Bytes> + std::fmt::Debug,
    {
        Ok::<_, BoxError>(
            new_service_responder(
                &self.client,
                &self.name,
                &self.subject,
                boxed_future_generator(func),
                self.token.clone(),
            )
            .await?,
        )
    }

    #[instrument(skip(self, func), fields( kong_name = %self.name, kong_subject = %self.subject))]
    pub async fn service_future<O, T>(
        &self,
        func: fn(Frame) -> O,
        // func: fn() -> T,
    ) -> Result<Instrumented<tokio::task::JoinHandle<Result<(), BoxError>>>, BoxError>
    where
        O: Future<Output = Result<T, BoxError>> + Send + 'static,
        T: Into<Bytes> + std::fmt::Debug + Send + 'static,
    {
        Ok(new_service_future_responder::<O, T>(
            &self.client,
            &self.name,
            &self.subject,
            func,
            self.token.clone(),
        )
        .await?)
    }


    #[instrument(skip(self, service), fields( kong_name = %self.name, kong_subject = %self.subject))]
    pub async fn tower_service<'a, S>(
        &'a self,
        service: Arc<Mutex<S>>, // func: fn() -> T,
    ) -> Result<Instrumented<tokio::task::JoinHandle<Result<(), BoxError>>>, BoxError>
    where
        S: tower::Service<Frame> + Send + Sync + Clone + 'static,
        S::Future: Send + Sync,
        S::Response: Into<Bytes> + Send + Sync,
        S::Error: Into<BoxError>,
    {
        Ok(new_tower_service_responder::<S>(
            &self.client,
            &self.name,
            &self.subject,
            service,
            self.token.clone(),
        )
        .await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::make_request;

    #[tokio::test]
    async fn initialize_kong() -> Result<(), Error> {
        let _kong = Kong::new("greet.kong", "10.2.4.106:4222").await;
        assert!(true);
        Ok(())
    }

    #[tokio::test]
    async fn named_kong() -> Result<(), Error> {
        // let kong = Kong::new("greet", "nats://10.2.4.106:4222").await;
        // let listener = kong.listen().await?;
        // let client: async_nats::Client = new_client("nats://10.2.4.106:4222").await?;
        // let _request = make_request(
        //     client.clone(),
        //     "greet.sue".to_string(),
        //     "Hello! My name is Sue!".to_string(),
        // )
        // .await?;
        // listener.abort();
        assert!(false);
        Ok(())
    }
}
