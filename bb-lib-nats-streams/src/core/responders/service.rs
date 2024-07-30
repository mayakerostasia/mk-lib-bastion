use super::super::replies::{echo_request, reply_with_future, reply_with_object};
use crate::{util::BoxedFutureFn, Decoder, Frame, NSLibError};
use anyhow::anyhow;
use async_nats::HeaderMap;
use bytes::Bytes;
use futures::StreamExt;
use std::future::Future;
use tokio_util::sync::CancellationToken;
use tower::{BoxError, Service, ServiceExt};
use tracing::{debug, error, info, info_span, instrument, trace, Instrument};

pub type Error = crate::NSLibError;

#[instrument(skip_all, fields(health = "unset", kong_name = %name, kong_subject = %subject))]
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
    info!("Starting responder @ {name}");
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
                            info!(%request.subject, ?request.payload);
                            let result: T = func().await;
                            info!("Result is {:#?}", &result);
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
