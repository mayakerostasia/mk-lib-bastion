use bytes::Bytes;
use std::future::Future;
use crate::{Frame, frame_tools::encoder::Decoder};
use futures::StreamExt;
use tokio_util::sync::CancellationToken;
use tower::BoxError;
use tracing::{error, info, trace, debug};

pub async fn new_publish_subscriber<'a, T, O>(
    client: &'a async_nats::Client,
    name: &'a str,
    subject: &'a str,
    func: fn(Frame) -> O,
    cancel_token: CancellationToken,
) -> Result<tokio::task::JoinHandle<Result<(), BoxError>>, BoxError>
where
    O: Future<Output = Result<T, BoxError>> + Send + 'static,
    T: std::fmt::Debug + Into<Bytes> + Send,
{
    let mut subscription = client.clone().subscribe(subject.to_string()).await.unwrap();
    let handle = tokio::spawn( {
        // let client = client.clone();
        async move {
            let cancel_token = cancel_token.clone();
            let _cancel_token = cancel_token.clone();
            tokio::select! {
                _ = cancel_token.cancelled() => {
                        Ok::<(), BoxError>(())
                    },
                _ = async move {
                        let cancel = _cancel_token.clone();
                        while let Some(request) = subscription.next().await {
                            eprintln!("publish_subscriber:Message -> {}", request.subject);

                            if request.headers.is_some() {
                                let headers = request.headers.clone().unwrap();
                                let monkey_name = headers.get("monkey_name").expect("Header Name isn't 'monkey_name'");
                                eprintln!("from: {}", monkey_name);
                                trace!("from={}", monkey_name);
                            };
                            trace!(%request.subject, ?request.payload, "Decoding Frame");

                            let payload = request.payload.clone();
                            let frame: Frame = Frame::decode(&payload)?;

                            let result: Result<T, BoxError> = func(frame).await;
                            debug!("Result is {:#?}", &result);
                            eprintln!("service_responder:OK");
                            trace!("service_responder:OK");
                            match result {
                                Ok(_) => {
                                    trace!("Publish OK");
                                },
                                Err(e) => {
                                    cancel.cancel();
                                    error!("Error in Subscription Receiver -> {e:#?}");
                                }
                            };
                            
                    };
                    Ok::<(), BoxError>(())
                } => {
                    Ok::<(), BoxError>(())
                }
            }
        }
    });
    info!("responder listening to {name}.*");
    // .instrument(span);
    Ok(handle)
}

#[cfg(test)]
mod tests {
    use crate::Frame;
    use crate::{core::new_client, Monkey};

    use super::*;

    const NATS_ADDR: &str = "nats://10.2.4.106:4222";

    async fn frame_funk(frame: Frame) -> Result<Frame, BoxError> { 
        eprintln!("Frame received : {:#?}", frame);
        Ok::<_, BoxError> (Frame::Fin) 
    }

    #[tokio::test]
    async fn test_publish_subscriber() -> Result<(), BoxError> {
        let client = new_client(NATS_ADDR).await.unwrap();
        let _echo_responder = new_publish_subscriber(
            &client,
            "test-name",
            "test-echo",
            frame_funk,
            CancellationToken::new(),
        )
        .await?;
        let monkey = Monkey::new("test-echo", NATS_ADDR).await;
        monkey.publish(Frame::ping()).await?;
        // assert_eq((), pong);
        Ok(())
    }
}
