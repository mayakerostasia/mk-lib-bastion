use std::marker::PhantomData;

use bytes::Bytes;
use tower::{Layer, Service};

use crate::NatsSend;

pub struct NatsLayer<S, Request>
where
    S: Service<Request>,
    Request: Into<Bytes> + Clone + From<Bytes>,
{
    pub subject: String,
    pub nats_url: String,
    _phantom_service: PhantomData<S>,
    _phantom_request: PhantomData<Request>,
}

impl<S, Request> NatsLayer<S, Request>
where
    S: Service<Request>,
    Request: Into<Bytes> + Clone + From<Bytes>,
{
    pub fn new(subject: &str, nats_url: &str) -> Self {
        Self {
            subject: subject.to_string(),
            nats_url: nats_url.to_string(),
            _phantom_service: PhantomData,
            _phantom_request: PhantomData,
        }
    }
}

impl<S, Request> Layer<S> for NatsLayer<S, Request>
where
    S: Service<Request>,
    Request: Into<Bytes> + Clone + From<Bytes>,
{
    type Service = NatsSend<S, Request>;

    fn layer(&self, inner: S) -> Self::Service {
        NatsSend::new(self.subject.as_str(), self.nats_url.as_str(), inner)
    }
}
