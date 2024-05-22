mod bastion;
pub mod encoder;
pub mod frames;
mod responders;

use crate::Error;
use async_nats::{header, HeaderMap};
use bytes::Bytes;
use futures::StreamExt;
use std::env;
use tracing::{debug, info, info_span, instrument, instrument::Instrumented, trace, Instrument};

use crate::util::BoxedFutureFn;
use responders::{echo_request, reply_with_object};

#[instrument]
pub async fn new_echo_responder(
    client: &async_nats::Client,
    name: &str,
) -> Result<tokio::task::JoinHandle<Result<(), Error>>, Error> {
    let mut requests = client.subscribe(format!("{}.*", name)).await.unwrap();

    info!("Starting responder @ {name}");
    let handle = tokio::spawn({
        let client = client.clone();
        async move {
            while let Some(request) = requests.next().await {
                debug!("Request -> {:#?}", request);
                echo_request(request, &client).await?;
            }
            Ok::<(), Error>(())
        }
    });
    info!("listening to {name}.*");

    Ok(handle)
}

#[instrument(skip(object))]
pub async fn new_object_responder<T>(
    client: &async_nats::Client,
    name: &str,
    object: impl Into<Bytes>,
) -> Result<tokio::task::JoinHandle<Result<(), Error>>, Error> {
    let mut requests = client.subscribe(format!("{}.*", name)).await.unwrap();
    let object: Bytes = object.into();
    // let identity = annotate(|_req: &Bytes | { Box::new(object) });

    info!("Starting responder @ {name}");
    let handle = tokio::spawn({
        let client = client.clone();
        async move {
            while let Some(request) = requests.next().await {
                debug!("Request -> {:#?}", request);
                reply_with_object::<T>(request, &client, object.clone()).await?;
            }
            Ok::<(), Error>(())
        }
    });
    info!("responder listening to {name}.*");

    Ok(handle)
}

#[instrument(skip_all, fields(kong_name = %name, kong_subject = %subject))]
pub async fn new_service_responder<T: Send + std::fmt::Debug + Into<Bytes> + 'static>(
    client: &async_nats::Client,
    name: &str,
    subject: &str,
    func: BoxedFutureFn<T>,
) -> Result<tokio::task::JoinHandle<Result<(), Error>>, Error> {
    let mut requests = client
        .clone()
        .subscribe(format!("{}", subject))
        .await
        .unwrap();
    // let func = Arc::new(func);
    let handle = tokio::spawn({
        let client = client.clone();
        // let span = info_span!("Service Responder {}", &client.);
        async move {
            while let Some(request) = requests.next().await {
                info!("Request @ {}", &request.subject);
                debug!("Request -> {:#?}", request);
                trace!("Service calling Function");
                let result: T = func().await;
                debug!("Caller is {:#?}", &result);
                reply_with_object::<T>(request, &client, result).await?;
                // drop(span);
            }
            Ok::<(), Error>(())
        }
    });
    info!("Service Up! {name} listening to {subject}");
    Ok(handle)
}

#[instrument]
pub async fn new_client(url: &str) -> Result<async_nats::Client, Error> {
    debug!("Creating a new client for Nats @ {url}");
    let nats_url = env::var("NATS_URL").unwrap_or_else(|_| url.to_string());
    Ok(async_nats::connect(nats_url).await?)
}

#[instrument(skip(payload))]
pub async fn make_request(
    client: async_nats::Client,
    addr: String,
    payload: impl Into<Bytes>,
) -> Result<async_nats::Message, Error> {
    let response = client.clone().request(addr.clone(), payload.into()).await?;
    debug!("got a response: {:?}", &response);
    Ok(response)
}

#[instrument(skip(payload))]
pub async fn make_header_request(
    client: async_nats::Client,
    addr: String,
    payload: impl Into<Bytes>,
    headers: HeaderMap,
) -> Result<async_nats::Message, Error> {
    let response = client.clone().request_with_headers(addr.clone(), headers, payload.into()).await?;
    debug!("got a response: {:?}", &response);
    Ok(response)
}

#[cfg(test)]
mod tests {
    use crate::Error;
    use super::{make_request, new_client, new_echo_responder};
    use std::time::Duration;

    #[tokio::test]
    async fn test_make_request() -> Result<(), Error> {
        let _otel = rs_nico_tracing::initialize();
        let client: async_nats::Client = new_client("nats://10.2.4.106:4222").await?;

        let _responder = new_echo_responder(&client, "greet").await?;

        make_request(client.clone(), "greet.sue".to_string(), "".to_string()).await?;
        make_request(client.clone(), "greet.johnny".to_string(), "".to_string()).await?;
        make_request(client.clone(), "greet.bobby".to_string(), "".to_string()).await?;

        let response = tokio::time::timeout(
            Duration::from_millis(500),
            client.request("greet.bob", "".into()),
        )
        .await??;

        eprintln!("got a response 3: {:?}", &response.payload);

        Ok(())
    }
}
