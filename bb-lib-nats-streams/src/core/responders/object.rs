use super::super::replies::reply_with_object_headers;
// use super::super::replies::reply_with_object;
use bytes::Bytes;
use futures::StreamExt;
use tower::BoxError;
use tracing::trace;

// pub type Error = crate::NSLibError;

// #[instrument(skip_all, fields(health = "unset", kong_name = %name))]
pub async fn new_object_responder(
    client: &async_nats::Client,
    name: &str,
    subject: &str,
    object: impl Into<Bytes>,
) -> Result<tokio::task::JoinHandle<Result<(), BoxError>>, BoxError> {
    let mut requests = client.subscribe(subject.to_string()).await.unwrap();
    let object: Bytes = object.into();
    let mut headers = async_nats::HeaderMap::new();
    headers.insert("monkey_name", name);

    trace!("Starting responder @ {name}");
    let handle = tokio::spawn({
        let client = client.clone();
        let headers = headers.clone();
        async move {
            while let Some(request) = requests.next().await {
                eprintln!("object_responder:Request -> {}", request.subject);
                if request.headers.is_some() {
                    let headers = request.headers.clone().unwrap();
                    let monkey_name = headers.get("monkey_name").expect("Header Name isn't 'monkey_name'");
                    eprintln!("from: {}", monkey_name);
                    trace!("from={}", monkey_name);
                };
                trace!("Request -> {:#?}", request);
                reply_with_object_headers(request, &client, headers.clone(), object.clone())
                    .await?;
                eprintln!("object_responder:OK");
            }
            Ok::<(), BoxError>(())
        }
    });
    trace!("responder listening to {name}.*");

    Ok(handle)
}
