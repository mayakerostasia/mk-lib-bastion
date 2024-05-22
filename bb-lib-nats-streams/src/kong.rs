use crate::boxed_future_generator;
use crate::core::{
    new_client, new_echo_responder, new_object_responder, new_service_responder,
};
// use crate::NSLibError;
use crate::Error;
use bytes::Bytes;
use petname::Generator;
use rand::thread_rng;
use tracing::instrument;

// pub type Error = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug)]
pub struct Kong {
    pub name: String,
    subject: String,
    client: async_nats::Client,
    // bastion: Option<Bastion>,
}

impl Kong {
    pub async fn new(subject: &str, nats_url: &str) -> Self {
        let mut rng = thread_rng();
        let name = petname::Petnames::default().generate(&mut rng, 2, "-").expect("Petname Failed");
        Kong {
            name,
            subject: subject.to_string(),
            client: new_client(nats_url).await.unwrap(),
            // bastion: None,
        }
    }

    pub fn client(&self) -> Result<async_nats::Client, Error> {
        Ok(self.client.clone())
    }
    pub async fn listen(
        &self,
    ) -> Result<tokio::task::JoinHandle<Result<(), Error>>, Error> {
        Ok(new_echo_responder(&self.client, &self.subject).await?)
    }

    pub async fn serve<T>(
        &self,
        object: impl Into<Bytes>,
    ) -> Result<tokio::task::JoinHandle<Result<(), Error>>, Error> {
        Ok(new_object_responder::<T>(&self.client, &self.subject, object).await?)
    }

    #[instrument(skip(self, func), fields( kong_name = %self.name, kong_subject = %self.subject))]
    pub async fn service<T, U>(
        &self,
        func: fn() ->T,
    ) -> Result<tokio::task::JoinHandle<Result<(), Error>>, Error>
    where
        T: Send + futures::Future<Output = U> + 'static,
        U: Send + 'static + Into<Bytes> + std::fmt::Debug
    {
        Ok(new_service_responder(&self.client, &self.name, &self.subject, boxed_future_generator(func)).await?)
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
        let kong = Kong::new("greet", "nats://10.2.4.106:4222").await;
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
