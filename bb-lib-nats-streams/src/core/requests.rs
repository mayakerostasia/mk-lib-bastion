use async_nats::HeaderMap;
use bytes::Bytes;
use tracing::{debug, info, instrument, trace};

type Error = crate::NSLibError;

#[instrument]
pub async fn new_client(url: &str) -> Result<async_nats::Client, anyhow::Error> {
    info!(url = url, "New Nats Client");
    // let nats_url = env::var("NATS_ADDR").unwrap_or_else(|_| url.to_string());
    Ok(async_nats::connect(url).await?)
}

#[instrument(skip(payload))]
pub async fn make_publish(
    client: async_nats::Client,
    addr: String,
    payload: impl Into<Bytes>
) -> Result<(), Error>
{
    client.publish(addr, payload.into()).await?;
    Ok(())
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

#[instrument(skip(payload, client, addr), fields(nats_addr = %addr))]
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

    debug!(
        "subject={} from={}",
        &addr,
        super::extractors::get_header_key(response.clone(), "monkey_name").unwrap()
    );

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
