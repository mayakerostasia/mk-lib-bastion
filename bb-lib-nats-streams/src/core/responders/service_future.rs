use super::super::replies::reply_with_future;
use crate::Frame;
use anyhow::anyhow;
use bytes::Bytes;
use futures::StreamExt;
use std::future::Future;
use tokio_util::sync::CancellationToken;
use tower::BoxError;
use tracing::debug;

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
    eprintln!("Starting service_future responder @ {name}");
    let handle = tokio::spawn({
        let client = client.clone();
        let cancel_token = cancel_token.clone();
        let _cancel_token = cancel_token.clone();
        async move {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    eprintln!("Cancel Token Popped");
                    Err::<(), BoxError>(anyhow!("Cancelled").into())
                }
                result = async move {
                        // let cancel = _cancel_token.clone();
                        while let Some(request) = requests.next().await {
                            eprintln!("subject={} payload={:#?}", request.subject, request.payload);
                            match reply_with_future(request, &client, func).await {
                                Ok(resp) => {
                                    eprintln!("Response is {:#?}", resp);
                                },
                                Err(e) => {
                                    eprintln!("Whoops! Error in Service -> {e:#?}");
                                    // cancel.cancel();
                                }
                            };
                        };
                        Ok::<(), BoxError>(())
                    } => {
                        eprintln!("Nico : Result is {:#?}", result);
                        eprintln!("Nico : Result is {:#?}", result);
                        Err::<(), BoxError>(anyhow!("Fuck!").into())
                    }
            }
        }
    });
    eprintln!("Returning handle");
    Ok(handle)
}

