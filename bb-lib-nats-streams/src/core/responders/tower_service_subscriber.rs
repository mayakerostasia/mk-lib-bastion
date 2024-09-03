// use super::super::replies::reply_with_object;
use crate::{core::replies::reply_with_object_headers, Decoder, Frame, NSLibError};
use anyhow::anyhow;
use bytes::Bytes;
use futures::StreamExt;
use tokio_util::sync::CancellationToken;
use tower::{BoxError, Service, ServiceExt};
use tracing::{error, trace, debug};

// #[instrument(skip_all, fields(health = "unset", kong_name = %name, kong_subject = %subject))]
pub async fn new_tower_service_subscriber<'a, S>(
    client: &'a async_nats::Client,
    name: &'a str,
    subject: &'a str,
    service: S,
    cancel_token: CancellationToken,
) -> Result<tokio::task::JoinHandle<Result<(), BoxError>>, BoxError>
where
    S: Clone + Service<Frame> + Send + Sync + 'static,
    S::Future: Send + Sync,
    S::Response: Send + Sync + Into<Bytes> + std::fmt::Debug,
    S::Error: Into<BoxError>,
{
    let mut requests = client.clone().subscribe(subject.to_string()).await.unwrap();
    let mut service = service.clone();
    trace!("Starting responder name={name} subject={subject}");
    let mut headers = async_nats::HeaderMap::new();
    headers.insert("monkey_name", name);

    let handle = tokio::spawn({
        let client = client.clone();
        let headers = headers.clone();
        let cancel_token = cancel_token.clone();
        let _cancel_token = cancel_token.clone();
        async move {
            let _srv = service.ready().await.map_err(Into::into)?;
            tokio::select! {
                _ = cancel_token.cancelled() => {
                        Err::<(), BoxError>(anyhow!("Cancelled").into())
                },
                result = async move {
                        let cancel = _cancel_token.clone();
                        let mutsrv = _srv.clone();
                        let headers = headers.clone();
                        while let Some(request) = requests.next().await {
                            eprintln!("tower_service_responder:Request -> {}", request.subject);
                            if request.headers.is_some() {
                                let headers = request.headers.clone().unwrap();
                                let monkey_name = headers.get("monkey_name").expect("Header Name isn't 'monkey_name'");
                                eprintln!("from: {}", monkey_name);
                                trace!("from={}", monkey_name);
                            };

                            let mut srv = mutsrv.clone();
                            let frame: Frame = match Frame::decode(&request.payload) {
                                Ok(fr) => {
                                    trace!("Frame Decoded");
                                    fr
                                },
                                Err(e) => {
                                    eprintln!("Unable to decode frame with error -> {e:#?}");
                                    cancel.cancel();
                                    return Err::<_, BoxError>(NSLibError::FrameDecodeError(format!("Whoops! Bad Frame! {:#?}", e).to_string()).into());
                                },
                            };

                            trace!("Msg Received -> {frame:#?}");
                            trace!("Readying Service");
                            let mut _srv = match srv.ready().await.map_err(Into::into) {
                                Ok(serv) => serv,
                                Err(e) => {
                                    eprintln!("Service couldn't ready up {:#?}", e);
                                    cancel.cancel();
                                    return Err::<_, BoxError>(NSLibError::ServiceReadyError(format!("Whoops! Service couldn't ready up {:#?}", e).to_string()).into())
                                },
                            };
                            trace!("Service Ready");

                            // Tower Service Subscriber does no Reply
                            let new_frame: <S as Service<Frame>>::Response = match _srv.call(frame).await.map_err(Into::into) {
                                Ok(fr) => fr,
                                Err(e) => {
                                    error!("Service Failed to call {:#?}", e);
                                    cancel.cancel();
                                    return Err::<_, BoxError>(NSLibError::NatsError(e).into())
                                },
                            };

                            trace!("Service call completed ret_frame={new_frame:#?}");
                            debug!("Subscriber ret frame -> {:#?}", new_frame);

                            // Tower Service Subscriber does not Reply
                            // TODO: Clean me up
                            match reply_with_object_headers(request, &client, headers.clone(), new_frame).await {
                                Ok(reply) => { eprintln!("Reply : {:#?}", reply) },
                                Err(e) => {
                                    error!("ERROR: tower_service_responder -> {e:#?}");
                                    cancel.cancel();
                                    return Err::<_, BoxError>(NSLibError::NatsError(Box::new(e)).into())
                                }
                            };

                            eprintln!("tower_service_responder:OK");
                            trace!("tower_service_responder:OK");
                        };
                        Ok::<(), BoxError>(())
                } => {
                    error!("EXITING : Result is {:#?}", result);
                    Ok::<(), BoxError>(())
                }
            }
        }
    });
    // .instrument(span);
    //
    // todo!("Finish this shit ")
    Ok(handle)
}

#[cfg(test)]
mod tests {
    use crate::Decoder;
    use std::future::Future;
    use std::pin::Pin;
    use std::task::Context;
    use std::task::Poll;

    use crate::Frame;
    use crate::{core::new_client, Monkey};

    use super::*;

    const NATS_ADDR: &str = "nats://10.2.4.106:4222";

    use tower::{BoxError, Service};

    #[derive(Clone)]
    struct MockService;

    impl Service<Frame> for MockService {
        type Response = Frame;
        type Error = BoxError;
        type Future =
            Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + Sync>>;

        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, req: Frame) -> Self::Future {
            let fut = async move {
                // Process the request and return a response
                println!("Mock-> Req -> {:#?}", req);
                Ok::<_, BoxError>(req)
            };
            Box::pin(fut)
        }
    }

    #[tokio::test]
    async fn test_tower_service_subscriber() -> Result<(), BoxError> {
        let client = new_client(NATS_ADDR).await.unwrap();
        let _echo_responder = new_tower_service_subscriber(
            &client,
            "test-tower-service",
            "test-tower-service",
            MockService {},
            CancellationToken::new(),
        )
        .await?;
        let monkey = Monkey::new("test-tower-service", NATS_ADDR).await;
        let _pong = monkey.publish(Frame::ping()).await?;
        // let pong_frame = Frame::decode(&pong.payload).unwrap();
        // assert_eq!(Frame::ping(), pong_frame);
        Ok(())
    }
}
