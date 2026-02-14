use crate::Decoder;
use crate::Frame;
// use anyhow::anyhow;
use bytes::Bytes;
use std::future::Future;
use tower::BoxError;
use tracing::{debug, error, info, trace};

type Error = crate::NSLibError;

pub async fn echo_request(
    request: async_nats::Message,
    client: &async_nats::Client,
) -> Result<(), Error> {
    let name = &request.subject.clone();
    let payload = &request.payload;
    info!(nats_subject = %name, monkey_payload = ?payload, "Msg Received from {}", &name);

    if let Some(reply) = request.reply {
        client.publish(reply, payload.clone()).await?;
    }
    Ok(())
}

pub async fn reply_with_object_headers(
    request: async_nats::Message,
    client: &async_nats::Client,
    headers: async_nats::HeaderMap,
    object: impl Into<Bytes>,
) -> Result<(), Error> {
    trace!("reply_with_object_headers() ->{request:#?}");
    if let Some(reply) = request.reply {
        trace!("Trying publish");
        client
            .publish_with_headers(reply, headers, object.into())
            .await?;
    } else {
        debug!("No Reply");
    }
    trace!("Reply Finished");
    Ok(())
}

pub async fn reply_with_future<O, T>(
    request: async_nats::Message,
    client: &async_nats::Client,
    fut: fn(Frame) -> O,
) -> Result<(), BoxError>
where
    O: Future<Output = Result<T, BoxError>> + Send,
    T: std::fmt::Debug + Into<Bytes> + Send,
{
    trace!("Starting Reply");

    // let name = request.subject.clone();
    let payload = request.payload.clone();
    let frame: Frame = Frame::decode(&payload)?;
    let resp = fut(frame).await?;
    if let Some(reply) = request.reply {
        client.publish(reply, resp.into()).await?;
    } else {
        error! {"No Reply in request"};
    }

    trace!("Reply Finished");
    Ok(())
}
