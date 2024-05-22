use crate::Error;
use bytes::Bytes;
use tracing::{debug, info, instrument};

#[instrument(skip(request, client))]
pub async fn echo_request(
    request: async_nats::Message,
    client: &async_nats::Client,
) -> Result<(), Error> {
    let name = &request.subject.clone();
    let payload = &request.payload;
    info!("Received from {}", &name);
    info!("Received payload: {:#?}", &payload);

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

#[instrument(skip(request, client, object))]
pub async fn reply_with_object<T>(
    request: async_nats::Message,
    client: &async_nats::Client,
    object: impl Into<Bytes>,
) -> Result<(), Error> {
    let name = &request.subject.clone();
    let payload = &request.payload.clone();
    info!("Received from {}", &name);
    info!("Received payload: {:#?}", &payload);
    if let Some(reply) = request.reply {
        client.publish(reply, ret_object::<T>(object)).await?;
    }
    Ok(())
}

// type SomeFuture = <Bastion as Service<BastionRequest>>::Future;
// #[instrument(skip(request, client, func))]
// pub async fn reply_with_future(
//     request: async_nats::Message,
//     client: &async_nats::Client,
//     func: Arc<BoxedFutureFn>,
// ) -> Result<(), async_nats::Error> {
//     let name = &request.subject.clone();
//     let payload = &request.payload.clone();
//     let future = func().await?;
//     info!("Received from {}", &name);
//     info!("Received payload: {:#?}", &payload);
//     if let Some(reply) = request.reply {
//         client.publish(reply, future.into()).await?;
//     }
//     Ok(())
// }
