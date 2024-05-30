use async_nats::HeaderMap;
use bytes::Bytes;
use futures::StreamExt;
// pub use operations::match_frame;
use std::{env, sync::Arc, future::Future};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, info_span, instrument, instrument::Instrumented, Instrument};

use crate::{util::BoxedFutureFn, Decoder, Frame};
use super::replies::{echo_request, reply_with_object, reply_with_future};
use tower::{BoxError, Service, ServiceExt};

// mod replies;

// pub use future::FrameFuture;

pub type Error = crate::NSLibError;


#[instrument]
pub async fn new_echo_responder(
    client: &async_nats::Client,
    name: &str,
) -> Result<tokio::task::JoinHandle<Result<(), Error>>, Error> {
    let subscribe = client.subscribe(format!("{}.*", name)).await.unwrap();
    let mut requests = subscribe;

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
        .instrument(info_span!("Kong"))
    });
    info!("listening to {name}.*");

    Ok(handle)
}

#[instrument(skip(object))]
pub async fn new_object_responder<T>(
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
                reply_with_object::<T>(request, &client, object.clone()).await?;
            }
            Ok::<(), BoxError>(())
        }
    });
    info!("responder listening to {name}.*");

    Ok(handle)
}

#[instrument(skip_all, fields(health = "unset", kong_name = %name, kong_subject = %subject))]
pub async fn new_service_responder<T: Send + std::fmt::Debug + Into<Bytes> + 'static>(
    client: &async_nats::Client,
    name: &str,
    subject: &str,
    func: BoxedFutureFn<T>,
    cancel_token: CancellationToken,
) -> Result<Instrumented<tokio::task::JoinHandle<Result<(), BoxError>>>, BoxError> {
    let mut requests = client
        .clone()
        .subscribe(format!("{}", subject))
        .await
        .unwrap();
    // let func = Arc::new(func);
    let span = info_span!("ServiceResponder");
    let handle = tokio::spawn({
        let client = client.clone();
        async move {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                        Ok::<(), BoxError>(())
                    },

                _ = async move {
                    while let Some(request) = requests.next().await {
                        info!(?request.subject, ?request.payload);
                        let result: T = func().await;
                        debug!("Result is {:#?}", &result);
                        reply_with_object::<T>(request, &client, result).await?;
                        };
                        Ok::<(), BoxError>(())
                    }
                    => {
                        Ok::<(), BoxError>(())
                    }
            }
        }
    })
    .instrument(span);
    Ok(handle)
}

#[instrument(skip_all, fields(health = "unset", kong_name = %name, kong_subject = %subject))]
pub async fn new_service_future_responder<O, T>(
    client: &async_nats::Client,
    name: &str,
    subject: &str,
    // func: fn(Frame) -> T,
    // func: BoxedFutureFn<T>,
    func: fn(Frame) -> O,
    cancel_token: CancellationToken,
) -> Result<Instrumented<tokio::task::JoinHandle<Result<(), BoxError>>>, BoxError>
where
    T: std::fmt::Debug + Into<Bytes> + Send + 'static,
    O: Future<Output = Result<T, BoxError>> + Send + 'static,
{
    let mut requests = client
        .clone()
        .subscribe(format!("{}", subject))
        .await
        .unwrap();
    // let func = Arc::new(func);
    let span = info_span!("ServiceResponder");
    let handle = tokio::spawn({
        let client = client.clone();
        // let func = Box::new(func);
        async move {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                        Ok::<(), BoxError>(())
                    },

                _ = async move {
                    while let Some(request) = requests.next().await {
                        info!(?request.subject, ?request.payload);
                        reply_with_future(request, &client, func).await?;
                        };
                        Ok::<(), BoxError>(())
                    }// .instrument(info_span!("request"))
                    => {
                        Ok::<(), BoxError>(())

                    }
            } // .instrument(info_span!("select"))
        } // .instrument(info_span!("async"))
    })
    .instrument(span);
    Ok(handle)
}

// use std::sync::Arc;
use tokio::sync::Mutex;
#[instrument(skip_all, fields(health = "unset", kong_name = %name, kong_subject = %subject))]
pub async fn new_tower_service_responder<'a, S>(
    client: &'a async_nats::Client,
    name: &'a str,
    subject: &'a str,
    service: Arc<Mutex<S>>,
    cancel_token: CancellationToken,
) -> Result<Instrumented<tokio::task::JoinHandle<Result<(), BoxError>>>, Error>
where
    S: Clone + Service<Frame> + Send + Sync + 'static,
    S::Future: Send + Sync,
    S::Response: Send + Sync + Into<Bytes>,
    S::Error: Into<BoxError>,
{
    let mut requests = client
        .clone()
        .subscribe(format!("{}", subject))
        .await
        .unwrap();
    // let func = Arc::new(func);
    let span = info_span!("ServiceResponder");

    let handle = tokio::spawn({
        let client = client.clone();
        // let service = service.clone();
        // let func = Box::new(func);
        async move {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                        Ok::<(), BoxError>(())
                    },
                _ = async move {
                        let mut _srv = service.lock().await;
                        let srv = _srv.ready().await.map_err(Into::into)?;
                        while let Some(request) = requests.next().await {
                            let srv = srv.ready().await.map_err(Into::into)?;
                            info!(?request.subject, ?request.payload);
                            let fut = srv.call(Frame::decode(&request.payload)).await.map_err(Into::into)?;
                            reply_with_object::<Frame>(
                                request,
                                &client,
                                fut
                            ).await?;
                        };
                    Ok::<(), BoxError>(())
                } => { Ok::<(), BoxError>(()) }

            }
        }
    }).instrument(span);
    //
    // todo!("Finish this shit ")
    Ok(handle)
}

#[instrument]
pub async fn new_client(url: &str) -> Result<async_nats::Client, anyhow::Error> {
    info!(url = url, "New Nats Client");
    let nats_url = env::var("NATS_ADDR").unwrap_or_else(|_| url.to_string());
    Ok(async_nats::connect(nats_url).await?)
}

#[instrument(skip(payload))]
pub async fn make_request(
    client: async_nats::Client,
    addr: String,
    payload: impl Into<Bytes>,
) -> Result<async_nats::Message, Error> {
    let response = client
        .clone()
        .request(addr.clone(), payload.into())
        .await
        .map_err(|e| Error::RequestError(e))?;
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
    let response = client
        .clone()
        .request_with_headers(addr.clone(), headers, payload.into())
        .await
        .map_err(|e| Error::RequestError(e))?;
    debug!("got a response: {:?}", &response);
    Ok(response)
}

// TODO: Remake test - This one runs forever
// #[cfg(test)]
// mod tests {
//     use crate::Error;
//     use super::{make_request, new_client, new_echo_responder};
//     use std::time::Duration;

//     #[tokio::test]
//     async fn test_make_request() -> Result<(), Error> {
//         let _otel = bb_lib_tracing::initialize();
//         let client: async_nats::Client = new_client("nats://10.2.4.106:4222").await?;

//         let _responder = new_echo_responder(&client, "greet").await?;

//         make_request(client.clone(), "greet.sue".to_string(), "".to_string()).await?;
//         make_request(client.clone(), "greet.johnny".to_string(), "".to_string()).await?;
//         make_request(client.clone(), "greet.bobby".to_string(), "".to_string()).await?;

//         // let response = tokio::time::timeout(
//         //     Duration::from_millis(500),
//         //     client.request("greet.bob", "".into()),
//         // ).await??;

//         // eprintln!("got a response 3: {:?}", &response.payload);

//         Ok(())
//     }
// }
