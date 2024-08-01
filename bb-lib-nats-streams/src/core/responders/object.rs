use super::super::replies::reply_with_object_headers;
// use super::super::replies::reply_with_object;
use bytes::Bytes;
use futures::StreamExt;
use tower::BoxError;
use tracing::{debug, info, instrument};

// pub type Error = crate::NSLibError;

#[instrument(skip_all, fields(health = "unset", kong_name = %name))]
pub async fn new_object_responder(
    client: &async_nats::Client,
    name: &str,
    object: impl Into<Bytes>,
) -> Result<tokio::task::JoinHandle<Result<(), BoxError>>, BoxError> {
    let mut requests = client.subscribe(format!("{}.*", name)).await.unwrap();
    let object: Bytes = object.into();
    let mut headers = async_nats::HeaderMap::new();
    headers.insert("monkey_name", name);

    info!("Starting responder @ {name}");
    let handle = tokio::spawn({
        let client = client.clone();
        let headers = headers.clone();
        async move {
            while let Some(request) = requests.next().await {
                debug!("Request -> {:#?}", request);
                reply_with_object_headers(request, &client, headers.clone(), object.clone()).await?;
            }
            Ok::<(), BoxError>(())
        }
    });
    info!("responder listening to {name}.*");

    Ok(handle)
}
