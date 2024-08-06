use super::super::replies::reply_with_future;
use crate::Frame;
use anyhow::anyhow;
use bytes::Bytes;
use futures::StreamExt;
use std::future::Future;
use tokio_util::sync::CancellationToken;
use tower::BoxError;
use tracing::{error, trace, warn};

// type Error = crate::NSLibError;

// #[instrument(skip_all, fields(kong_name = %name, kong_subject = %subject))]
pub async fn new_service_future_responder<O, T>(
    client: &async_nats::Client,
    name: &str,
    subject: &str,
    func: fn(Frame) -> O,
    cancel_token: CancellationToken,
) -> Result<tokio::task::JoinHandle<Result<(), BoxError>>, BoxError>
where
    T: std::fmt::Debug + Into<Bytes> + Send,
    O: Future<Output = Result<T, BoxError>> + Send + 'static,
{
    let mut requests = client.clone().subscribe(subject.to_string()).await.unwrap();
    trace!("Starting service_future responder @ {name}");
    let handle = tokio::spawn({
        let client = client.clone();
        let cancel_token = cancel_token.clone();
        let _cancel_token = cancel_token.clone();
        async move {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    warn!("Cancel Token Popped");
                    Err::<(), BoxError>(anyhow!("Cancelled").into())
                }
                result = async move {
                        // let cancel = _cancel_token.clone();
                        while let Some(request) = requests.next().await {
                            eprintln!("service_future_responder:Request -> {:#?}", request);
                            trace!("subject={} payload={:#?}", request.subject, request.payload);
                            match reply_with_future(request, &client, func).await {
                                Ok(resp) => {
                                    trace!("Response is {:#?}", resp);
                                },
                                Err(e) => {
                                    error!("Whoops! Error in Service -> {e:#?}");
                                    // cancel.cancel();
                                }
                            };
                            eprintln!("service_future_responder:OK")
                        };
                        Ok::<(), BoxError>(())
                    } => {
                        error!("ERROR: service_future -> {:#?}", result);
                        Err::<(), BoxError>(anyhow!("Fuck!").into())
                    }
            }
        }
    });
    trace!("Returning handle");
    Ok(handle)
}

#[cfg(test)]
mod tests {
    use crate::Decoder;
    use crate::Frame;
    use crate::{core::new_client, Monkey};

    use super::*;

    const NATS_ADDR: &str = "nats://10.2.4.106:4222";

    async fn frame_funk(_frame: Frame) -> Result<Frame, BoxError> {
        Ok(Frame::pong())
    }

    #[tokio::test]
    async fn test_service_future_responder() -> Result<(), BoxError> {
        let client = new_client(NATS_ADDR).await.unwrap();
        let _echo_responder = new_service_future_responder(
            &client,
            "test-name",
            "test-echo",
            frame_funk,
            CancellationToken::new(),
        )
        .await?;
        let monkey = Monkey::new("test-echo", NATS_ADDR).await;
        let pong = monkey.msg(Frame::ping()).await?;
        let pong_frame = Frame::decode(&pong.payload).unwrap();
        assert_eq!(Frame::pong(), pong_frame);
        Ok(())
    }
}
