use super::replies::{echo_request, reply_with_future, reply_with_object};
use crate::{util::BoxedFutureFn, Decoder, Frame, NSLibError};
use async_nats::HeaderMap;
use bytes::Bytes;
use futures::StreamExt;
use std::future::Future;
use tokio_util::sync::CancellationToken;
use tower::{BoxError, Service, ServiceExt};
use tracing::{debug, error, info, info_span, instrument, trace, Instrument};

pub type Error = crate::NSLibError;

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
                trace!("Request -> {:#?}", request);
                echo_request(request, &client).await?;
            }
            Ok::<(), Error>(())
        }
        .instrument(info_span!("Kong"))
    });
    info!("listening to {name}.*");

    Ok(handle)
}

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
    // let func = Arc::new(func);
    // let span = trace_span!("ServiceResponder");
    info!("Starting responder @ {name}");
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
                            info!("Result is {:#?}", &result);
                            reply_with_object(request, &client, result).await?;
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

// #[instrument(skip_all, fields(kong_name = %name, kong_subject = %subject))]
pub async fn new_service_future_responder<O, T>(
    client: &async_nats::Client,
    name: &str,
    subject: &str,
    // func: fn(Frame) -> T,
    // func: BoxedFutureFn<T>,
    func: fn(Frame) -> O,
    cancel_token: CancellationToken,
) -> Result<tokio::task::JoinHandle<Result<(), BoxError>>, BoxError>
where
    T: std::fmt::Debug + Into<Bytes> + Send,
    O: Future<Output = Result<T, BoxError>> + Send + 'static,
{
    let mut requests = client.clone().subscribe(subject.to_string()).await.unwrap();
    // let func = Arc::new(func);
    // let span = info_span!("ServiceResponder");
    info!("Starting responder @ {name}");
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
                    }
                    => {
                        Ok::<(), BoxError>(())

                    }
            }
        }
    });
    // .instrument(span);
    Ok(handle)
}

// #[instrument(skip_all, fields(health = "unset", kong_name = %name, kong_subject = %subject))]
pub async fn new_tower_service_responder<'a, S>(
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
    // let span = info_span!("ServiceResponder");
    let mut service = service.clone();
    info!("Starting responder @ {name}");
    let handle = tokio::spawn({
        let client = client.clone();
        // let service = service.clone();
        // let func = Box::new(func);
        async move {
            let _srv = service.ready().await.map_err(Into::into)?;
            tokio::select! {
                _ = cancel_token.cancelled() => {
                        Ok::<(), BoxError>(())
                    },
                result = async move {
                        let mutsrv = _srv.clone();
                        while let Some(request) = requests.next().await {
                            let mut srv = mutsrv.clone();
                            let frame: Frame = match Frame::decode(&request.payload) {
                                Ok(fr) => fr,
                                Err(e) => {
                                    error!("Unable to decode frame with error -> {e:#?}");
                                    return Err::<_, BoxError>(NSLibError::FrameDecodeError(format!("Whoops! Bad Frame! {:#?}", e).to_string()).into());
                                },
                            };
                            info!("Msg Received -> {frame:#?}");
                            trace!("Frame Decoded - Calling Service");

                            trace!("Readying Service");
                            let mut _srv = match srv.ready().await.map_err(Into::into) {
                                Ok(serv) => serv,
                                Err(e) => return Err::<_, BoxError>(NSLibError::ServiceReadyError(format!("Whoops! Service couldn't ready up {:#?}", e).to_string()).into()),
                            };
                            trace!("Service Ready");

                            let new_frame: <S as Service<Frame>>::Response = match _srv.call(frame).await.map_err(Into::into) {
                                Ok(fr) => fr,
                                Err(e) => return Err::<_, BoxError>(NSLibError::NatsError(e).into()),
                            };

                            trace!("Service call completed - Composing Reply");

                            match reply_with_object(request, &client, new_frame).await {
                                Ok(reply) => { info!(?reply) },
                                Err(e) => { error!(?e) }
                            };

                            trace!("Replied with object");
                        };
                        Ok::<(), BoxError>(())

                } => {
                    eprintln!("Nico HEY: Result is {:#?}", result);
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

// #[instrument]
pub async fn new_client(url: &str) -> Result<async_nats::Client, anyhow::Error> {
    info!(url = url, "New Nats Client");
    // let nats_url = env::var("NATS_ADDR").unwrap_or_else(|_| url.to_string());
    Ok(async_nats::connect(url).await?)
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
    trace!("got a response: {:?}", &response);
    eprintln!("Response is {:#?}", &response);
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

    // debug!(
    //     "subject={} from={}",
    //     addr.clone(),
    //     response
    //         .headers
    //         .
    //         // .expect("getting headers")
    //         // .get("monkey_name")
    //         // .expect("getting monkey name")
    // );

    trace!("got a response: {:?}", &response);
    Ok(response)
}

#[instrument(skip(payload))]
pub async fn make_timeout_header_request(
    client: async_nats::Client,
    addr: String,
    payload: impl Into<Bytes>,
    timeout: Option<core::time::Duration>,
    headers: HeaderMap,
) -> Result<async_nats::Message, Error> {
    let request = async_nats::Request::new()
        // .inbox(format!("monkey@{}", addr))
        .timeout(timeout)
        .headers(headers)
        .payload(payload.into());

    let response = client
        .clone()
        .send_request(addr.clone(), request)
        .await
        .map_err(Error::RequestError)?;

    debug!(
        "subject={} from={}",
        addr.clone(),
        super::extractors::get_header_key(response.clone(), "monkey_name").unwrap()
    );

    trace!("Nats Response is: {:?}", &response);
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
    trace!("got a response: {:?}", &response);
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
