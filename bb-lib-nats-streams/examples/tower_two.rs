use bb_lib_nats_streams::NatsSend;
use bytes::Bytes;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{BoxError, Service, ServiceExt};

#[derive(Clone)]
struct MockService;

impl Service<Bytes> for MockService {
    type Response = Bytes;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + Sync>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Bytes) -> Self::Future {
        let fut = async move {
            // Process the request and return a response
            Ok(req)
        };
        Box::pin(fut)
    }
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let subject = "test_subject";
    let nats_url = "nats://10.2.4.106:4222";
    let msg = Bytes::from("test_message");
    let inner_service = MockService;

    eprintln!("0");
    let mut nats_send = NatsSend::new(subject, nats_url, msg.clone(), inner_service.clone());
    eprintln!("1");
    let sender = nats_send.ready().await?;
    eprintln!("2");

    // Test call
    let response = sender.call(msg.clone()).await.unwrap();
    eprintln!("3");
    // assert_eq!(response, msg);
    Ok(())
}
