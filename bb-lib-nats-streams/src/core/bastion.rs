use bytes::Bytes;
use crate::{Decoder, Encoder};
use tower::Service;
use futures::Future;
use serde::{Deserialize, Serialize};
use std::task::Poll;
use std::pin::Pin;
use tracing::debug;

#[derive(Debug, Serialize, Deserialize)]
pub struct BastionRequest(pub async_nats::Message);

impl Encoder for BastionRequest {}
impl Decoder<'_, BastionRequest> for BastionRequest {}

impl Into<Bytes> for BastionRequest {
    fn into(self) -> Bytes {
        let encoded = self.encode();
        Bytes::from(encoded)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BastionReply;

impl Encoder for BastionReply {}
impl Decoder<'_, BastionReply> for BastionReply {}
impl Into<Bytes> for BastionReply {
    fn into(self) -> Bytes {
        let encoded = self.encode();
        Bytes::from(encoded)
    }
}

#[derive(Clone)]
pub struct Bastion {
    // func: Incrementer
}

impl Service<BastionRequest> for Bastion {
    type Response = BastionReply;
    type Error = async_nats::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: BastionRequest) -> Self::Future {
        debug!("Request is {req:#?}");
        Box::pin(async { Ok(BastionReply) })
    }
}
