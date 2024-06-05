use bb_lib_nats_streams::{Decoder, Encoder, Frame, NatsSend};
use bytes::Bytes;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{BoxError, Service, ServiceExt};

#[derive(Clone)]
struct MockService;

impl<T> Service<T> for MockService 
where 
    T: Into<Bytes> + From<Bytes> + Send + Sync + 'static
{
    type Response = T;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + Sync>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: T) -> Self::Future {
        let fut = async move {
            // Process the request and return a response
            eprintln!("Request has returned");
            Ok(req)
        };
        Box::pin(fut)
    }
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let subject = "gc-api.log";
    let nats_url = "nats://10.2.4.106:4222";
    let msg = Frame::message("Hello From Tower_Two!");
    let inner_service = MockService;
    let mut nats_send = NatsSend::new(subject, nats_url, inner_service.clone());
    let sender = nats_send.ready().await?;
    let response = sender.call(msg.clone()).await?;
    assert_eq!(response.encode(), msg.encode());
    Ok(())
}
