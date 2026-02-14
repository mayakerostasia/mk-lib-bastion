use crate::{core::replies::reply_with_object_headers, util::BoxedFutureFn};
use bytes::Bytes;
use futures::StreamExt;
use tokio_util::sync::CancellationToken;
use tower::BoxError;
use tracing::{error, info, trace};

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
    let mut headers = async_nats::HeaderMap::new();
    headers.insert("monkey_name", name);
    info!("Starting service responder @ {name}");

    let handle = tokio::spawn({
        let client = client.clone();
        let headers = headers.clone();
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
                            eprintln!("service_responder:Request -> {}", request.subject);
                            if request.headers.is_some() {
                                let headers = request.headers.clone().unwrap();
                                let monkey_name = headers.get("monkey_name").expect("Header Name isn't 'monkey_name'");
                                eprintln!("from: {}", monkey_name);
                                trace!("from={}", monkey_name);
                            };
                            trace!(%request.subject, ?request.payload);
                            let result: T = func().await;
                            trace!("Result is {:#?}", &result);
                            match reply_with_object_headers(request, &client, headers.clone(), result).await {
                                Ok(resp) => {
                                    trace!("Response is {:#?}", resp);
                                },
                                Err(e) => {
                                    error!("Whoops! Error in Service -> {e:#?}");
                                    cancel.cancel();
                                }
                            };
                            eprintln!("service_responder:OK");
                            trace!("service_responder:OK");
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
    use crate::util::boxed_future_generator;
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
