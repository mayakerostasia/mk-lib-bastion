use std::str::FromStr;

use crate::core::{make_header_request, make_request, make_timeout_request, new_client};
use crate::{Error};
use async_nats::{HeaderMap, HeaderName, HeaderValue};
use bytes::Bytes;
use petname::Generator;
use rand::thread_rng;
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct Monkey {
    pub name: String,
    pub subject: String,
    headers: async_nats::HeaderMap,
    _client: async_nats::Client,
}

// impl Borrow<Monkey> for Monkey {
//     fn borrow(&self) -> &Monkey {
//         self
//     }
// }
//
impl AsRef<Monkey> for Monkey {
    fn as_ref(&self) -> &Monkey {
        self
    }
}

impl AsMut<Monkey> for Monkey {
    fn as_mut(&mut self) -> &mut Monkey {
        self
    }
}

impl Monkey {
    pub async fn new(subject: &str, nats_url: &str) -> Self {
        let mut rng = thread_rng();
        let name = petname::Petnames::default()
            .generate(&mut rng, 2, "-")
            .expect("Petname Failed");
        Monkey {
            name,
            subject: subject.to_string(),
            headers: HeaderMap::new(),
            _client: new_client(nats_url).await.unwrap(),
        }
    }

    pub fn client(&self) -> Result<async_nats::Client, Error> {
        Ok(self._client.clone())
    }

    pub fn set_subject(&mut self, subject: &str) ->Result<(), Error> {
        self.subject = subject.to_string();
        Ok(())
    }

    pub fn set_header(&mut self, key: &str, val: &str) -> Result<(), Error> {
        let name: HeaderName = HeaderName::from_str(key)?;
        let value: HeaderValue = HeaderValue::from_str(val)?;
        self.headers.insert(name, value);
        Ok(())
    }

    #[instrument(skip(payload, self), fields(monkey_name = %self.name, monkey_subject = %self.subject))]
    pub async fn msg_timeout(
        &self,
        payload: impl Into<Bytes>,
        timeout: Option<std::time::Duration>,
    ) -> Result<async_nats::Message, Error> {
        Ok(make_timeout_request(
            self.client()?,
            self.subject.to_string(),
            payload.into(),
            timeout,
        )
        .await?)
    }

    #[instrument(skip(payload, self), fields(monkey_name = %self.name, monkey_subject = %self.subject))]
    pub async fn msg(&self, payload: impl Into<Bytes>) -> Result<async_nats::Message, Error> {
        Ok(make_request(self.client()?, self.subject.to_string(), payload.into()).await?)
    }

    #[instrument(skip(payload, self), fields(monkey_name = %self.name, monkey_subject = %self.subject))]
    pub async fn hmsg(&self, payload: impl Into<Bytes>) -> Result<async_nats::Message, Error> {
        Ok(make_header_request(
            self.client()?,
            self.subject.to_string(),
            payload.into(),
            self.headers.clone(),
        )
        .await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Error, Kong};

    const NATS_URL: &str = "nats://10.2.4.106:4222";

    #[tokio::test]
    async fn initialize_monkey() -> Result<(), Error> {
        let _kong = Kong::new("greet", NATS_URL).await;
        let _monkey = Monkey::new("greet.monkey", NATS_URL).await;
        // assert!(true);
        Ok(())
    }

    #[tokio::test]
    async fn named_kong() -> Result<(), Error> {
        let kong = Kong::new("greet", NATS_URL).await?;
        let listener = kong.listen().await?;

        let _client: async_nats::Client = new_client(NATS_URL).await?;
        let monkey = Monkey::new("greet.monkey", NATS_URL).await;
        eprintln!("Monkey: {:?} ", monkey);
        let _monkey_msg = monkey.msg("Hello!").await?;
        // let _request = make_request(client.clone(), "greet.sue".to_string(), "Hello! My name is Sue!".to_string()).await?;

        listener.abort();
        Ok(())
    }
}
