use super::super::replies::reply_with_object;
use crate::util::{BoxedFutureFn, boxed_future_generator};
use bytes::Bytes;
use futures::StreamExt;
use tokio_util::sync::CancellationToken;
use tower::BoxError;
use tracing::{error, info, instrument, trace, debug};

// pub type Error = crate::NSLibError;

// #[instrument(skip_all, fields(health = "unset", kong_name = %name, kong_subject = %subject))]
pub async fn new_service_responder<'a, T>(
    client: &'a async_nats::Client,
    name: &'a str,
    subject: &'a str,
    func: BoxedFutureFn<T>,
    cancel_token: CancellationToken,
) -> Result<tokio::task::JoinHandle<Result<(), BoxError>>, BoxError>
where
    T: Send + std::fmt::Debug + Into<Bytes> + 'static,
{
    let mut requests = client.clone().subscribe(subject.to_string()).await.unwrap();
    info!("Starting service responder @ {name}");
    let handle = tokio::spawn({
        let client = client.clone();
        async move {
            let cancel_token = cancel_token.clone();
            let _cancel_token = cancel_token.clone();
            tokio::select! {
                _ = cancel_token.cancelled() => {
                        Ok::<(), BoxError>(())
                    },

                _ = async move {
                        let cancel = _cancel_token.clone();
                        while let Some(request) = requests.next().await {
                            debug!(%request.subject, ?request.payload);
                            let result: T = func().await;
                            debug!("Result is {:#?}", &result);
                            match reply_with_object(request, &client, result).await {
                                Ok(resp) => {
                                    trace!("Response is {:#?}", resp);
                                },
                                Err(e) => {
                                    error!("Whoops! Error in Service -> {e:#?}");
                                    cancel.cancel();
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
    use crate::Decoder;
    use crate::Frame;
    use crate::{core::new_client, Monkey};

    use super::*;

    const NATS_ADDR: &str = "nats://10.2.4.106:4222";

    async fn frame_funk() -> Frame {
        Frame::pong()
    }

    #[tokio::test]
    async fn test_service_responder() -> Result<(), BoxError> {
        let client = new_client(NATS_ADDR).await.unwrap();
        let _echo_responder = new_service_responder(
            &client,
            "test-name",
            "test-echo",
            boxed_future_generator(frame_funk),
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
