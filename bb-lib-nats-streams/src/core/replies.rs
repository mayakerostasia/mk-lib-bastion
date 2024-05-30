use crate::Decoder;
use crate::Frame;
// use anyhow::anyhow;
use bytes::Bytes;
use std::future::Future;
use tower::BoxError;
use tracing::{debug, info, instrument};

type Error = crate::NSLibError;

#[instrument(skip(request, client), fields(monkey_name, monkey_payload))]
pub async fn echo_request(
    request: async_nats::Message,
    client: &async_nats::Client,
) -> Result<(), Error> {
    let name = &request.subject.clone();
    let payload = &request.payload;
    info!(monkey_name = %name, monkey_payload = ?payload, "Msg Received from {}", &name);

    if let Some(reply) = request.reply {
        client.publish(reply, payload.clone()).await?;
    }
    Ok(())
}

fn ret_object<'de, T>(object: impl Into<Bytes>) -> Bytes {
    let bytes: Bytes = object.into();
    debug!("Object to ret is {bytes:#?}");
    bytes
}

// #[instrument(skip(request, client, object))]
pub async fn reply_with_object<T>(
    request: async_nats::Message,
    client: &async_nats::Client,
    object: impl Into<Bytes>,
) -> Result<(), Error> {
    let name = &request.subject.clone();
    let payload = &request.payload;
    let headers = &request.headers;
    info!(subject = %name, payload = ?payload, ?headers, "Msg Received from {}", &name);
    if let Some(reply) = request.reply {
        client.publish(reply, ret_object::<T>(object)).await?;
    }
    Ok(())
}

// #[instrument(skip(request, client, fut))]
// pub async fn reply_with_future<F, T, E>(
//     request: async_nats::Message,
//     client: &async_nats::Client,
//     fut: fn(Frame) -> F,
// ) -> Result<(), E>
// where
//     F: Future<Output = Result<T, E>> + Send + 'static,
//     T: std::fmt::Debug + Into<Bytes> + Send + 'static,
//     E: std::error::Error + From<async_nats::client::PublishError> + Into<BoxError>,
// {
//     let name = request.subject.clone();
//     let payload = request.payload.clone();
//     let frame = Frame::decode(&payload);
//     info!(?name, ?payload, "Received payload: {:#?}", &frame);
//     let future = fut(frame.clone()).await?;
//     info!("Got result {:#?}", &future);
//     if let Some(reply) = request.reply {
//         client.publish(reply, future.into()).await?;
//     }
//     Ok(())
// }

#[instrument(skip(request, client, fut))]
pub async fn reply_with_future<O, T>(
    request: async_nats::Message,
    client: &async_nats::Client,
    fut: fn(Frame) -> O
) -> Result<(), BoxError> 
where
    O: Future<Output = Result<T, BoxError>> + Send + 'static,
    T: std::fmt::Debug + Into<Bytes> + Send + 'static,
{
    let name = request.subject.clone();
    let payload = request.payload.clone();
    let frame: Frame = Frame::decode(&payload);
    info!(?name, ?payload, "Received payload: {:#?}", &frame);
    let future = fut(frame).await?;
    info!("Got result {:#?}", &future);
    if let Some(reply) = request.reply {
        client.publish(reply, future.into()).await?;
    }
    Ok(())
}
