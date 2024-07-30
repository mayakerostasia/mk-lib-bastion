use super::super::replies::echo_request;
use futures::StreamExt;
use tracing::{info_span, instrument, Instrument};

type Error = crate::NSLibError;

#[instrument(skip_all, fields(health = "unset", kong_name = %name))]
pub async fn new_echo_responder(
    client: &async_nats::Client,
    name: &str,
) -> Result<tokio::task::JoinHandle<Result<(), Error>>, Error> {
    let subscribe = client.subscribe(format!("{}.*", name)).await.unwrap();
    let mut requests = subscribe;

    eprintln!("Starting responder @ {name}");
    let handle = tokio::spawn({
        let client = client.clone();
        async move {
            while let Some(request) = requests.next().await {
                eprintln!("Request -> {:#?}", request);
                echo_request(request, &client)
                    .instrument(info_span!("Echo Request"))
                    .await?;
            }
            Ok::<(), Error>(())
        }
    });
    eprintln!("listening to {name}.*");

    Ok(handle)
}

#[cfg(test)]
mod tests {
    use crate::Decoder;
    use crate::Frame;
    use crate::{core::new_client, Monkey};
    use anyhow::Error;

    use super::*;

    const NATS_ADDR: &str = "nats://10.2.4.106:4222";

    #[tokio::test]
    async fn test_echo_responder() -> Result<(), Error> {
        let client = new_client(NATS_ADDR).await.unwrap();
        let _echo_responder = new_echo_responder(&client, "test-echo").await?;
        let monkey = Monkey::new("test-echo.hi", NATS_ADDR).await;
        let pong = monkey.msg(Frame::ping()).await?;
        let pong_frame = Frame::decode(&pong.payload).unwrap();
        assert_eq!(Frame::ping(), pong_frame);
        Ok(())
    }
}
