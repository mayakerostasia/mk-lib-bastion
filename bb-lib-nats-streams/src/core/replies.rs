use crate::Decoder;
use crate::Frame;
use anyhow::anyhow;
// use anyhow::anyhow;
use bytes::Bytes;
use std::future::Future;
use tower::BoxError;
use tracing::{error, info, instrument, trace};

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

#[instrument(skip(request, client, object))]
pub async fn reply_with_object(
    request: async_nats::Message,
    client: &async_nats::Client,
    object: impl Into<Bytes>,
) -> Result<(), Error> {
    trace!("reply_with_object() ->{request:#?}");
    if let Some(reply) = request.reply {
        trace!("Trying publish");
        client.publish(reply, object.into()).await?;
    } else {
        error!("There's no Reply here");
        return Err(crate::NSLibError::Anyhow(anyhow!("No reply in request")));
    }
    trace!("Reply Finished");
    Ok(())
}

#[instrument(skip(request, client, fut))]
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
