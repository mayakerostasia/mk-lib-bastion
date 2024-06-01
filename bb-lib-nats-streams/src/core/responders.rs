use async_nats::HeaderMap;
use bytes::Bytes;
use futures::StreamExt;
// pub use operations::match_frame;
use std::{env, future::Future, sync::Arc};
use tokio_util::sync::CancellationToken;
use tracing::{
    debug, error, info, info_span, instrument, instrument::Instrumented, warn, Instrument,
};

use super::replies::{echo_request, reply_with_future, reply_with_object};
use crate::{util::BoxedFutureFn, Frame, Decoder};
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
pub async fn new_service_responder<'a, T>(
    client: &'a async_nats::Client,
    name: &'a str,
    subject: &'a str,
    func: BoxedFutureFn<T>,
    cancel_token: CancellationToken,
) -> Result<Instrumented<tokio::task::JoinHandle<Result<(), BoxError>>>, BoxError>
where
    T: Send + std::fmt::Debug + Into<Bytes> + 'static,
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
    T: std::fmt::Debug + Into<Bytes> + Send,
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

// use std::pin::Pin;
// async fn pin_future(
//     fut: impl Future<Output = Result<Frame, Error>> + Send + Sync + 'static,
// ) -> Pin<Box<dyn Future<Output = Result<Frame, Error>> + Send + Sync>> {
//     Box::pin(fut)
// }

// fn pop_call_tower_service(frame: Frame) {}

// use std::sync::Arc;
use tokio::sync::Mutex;
#[instrument(skip_all, fields(health = "unset", kong_name = %name, kong_subject = %subject))]
pub async fn new_tower_service_responder<'a, S>(
    client: &'a async_nats::Client,
    name: &'a str,
    subject: &'a str,
    service: Arc<Mutex<S>>,
    cancel_token: CancellationToken,
) -> Result<Instrumented<tokio::task::JoinHandle<Result<(), BoxError>>>, BoxError>
where
    S: Service<Frame> + Send + Sync + 'static,
    S::Future: Send + Sync,
    S::Response: Send + Sync + Into<Bytes> + std::fmt::Debug,
    S::Error: Into<BoxError>,
{
    let mut requests = client.clone().subscribe(subject.to_string()).await.unwrap();
    // let func = Arc::new(func);
    let span = info_span!("ServiceResponder");

    let handle = tokio::spawn({
        let client = client.clone();
        // let service = service.clone();
        // let service = service.clone();
        // let func = Box::new(func);
        async move {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                        Ok::<(), BoxError>(())
                    },
                _ = async move {
                        let service = service.clone();
                        while let Some(request) = requests.next().await {
                            let mut _srv_unlocked = service.lock().await;
                            let srv = _srv_unlocked.ready().await.map_err(Into::into)?;
                            info!(?request.subject, ?request.payload);
                            let frame: Frame = Frame::decode(&request.payload);
                            warn!(?frame);
                            let new_frame: <S as Service<Frame>>::Response = srv.call(frame).await.map_err(Into::into)?;
                            reply_with_object::<Frame>(request, &client, new_frame).await.map_err(Into::<BoxError>::into)?;
                        };
                        Ok::<(), BoxError>(())

                } => { Ok::<(), BoxError>(()) }

            }
        }
    })
    .instrument(span);
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
        .map_err(Error::RequestError)?;
    debug!("got a response: {:?}", &response);
    Ok(response)
}

#[instrument(skip(payload))]
pub async fn make_timeout_request(
    client: async_nats::Client,
    addr: String,
    payload: impl Into<Bytes>,
    timeout: Option<core::time::Duration>,
) -> Result<async_nats::Message, Error> {
    let request = async_nats::Request::new()
        // .inbox(format!("monkey@{}", addr))
        .timeout(timeout)
        .payload(payload.into());
    let response = client
        .clone()
        .send_request(addr.clone(), request)
        .await
        .map_err(Error::RequestError)?;
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
        .map_err(Error::RequestError)?;
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
