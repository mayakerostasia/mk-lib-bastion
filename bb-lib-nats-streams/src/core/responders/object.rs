use super::super::replies::reply_with_object;
use bytes::Bytes;
use futures::StreamExt;
use tower::BoxError;
use tracing::{info, instrument, debug};

// pub type Error = crate::NSLibError;

#[instrument(skip_all, fields(health = "unset", kong_name = %name))]
pub async fn new_object_responder(
    client: &async_nats::Client,
    name: &str,
    object: impl Into<Bytes>,
) -> Result<tokio::task::JoinHandle<Result<(), BoxError>>, BoxError> {
    let mut requests = client.subscribe(format!("{}.*", name)).await.unwrap();
    let object: Bytes = object.into();
    // let identity = annotate(|_req: &Bytes | { Box::new(object) });

    info!("Starting responder @ {name}");
    let handle = tokio::spawn({
        let client = client.clone();
        async move {
            while let Some(request) = requests.next().await {
                debug!("Request -> {:#?}", request);
                reply_with_object(request, &client, object.clone()).await?;
            }
            Ok::<(), BoxError>(())
        }
    });
    info!("responder listening to {name}.*");

    Ok(handle)
}

