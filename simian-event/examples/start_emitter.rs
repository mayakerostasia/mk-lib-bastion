use anyhow::Error;
use simian_nats_streams::KingKong;
use std::env;
use tracing::{info, info_span};

const BB_NATS_ADDR: &str = "nats://10.0.0.27:4222";
const BB_ENDPOINT: &str = "gc-api";
const BB_PATH: &str = "log";
const BB_HEALTHZ_BIND: &str = "0.0.0.0:6666";

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _otel = simian_tracing::initialize()?;
    let nats_addr = env::var("BB_NATS_ADDR").unwrap_or(BB_NATS_ADDR.to_string());
    let endpoint = env::var("BB_ENDPOINT").unwrap_or(BB_ENDPOINT.to_string());
    let path = env::var("BB_PATH").unwrap_or(BB_PATH.to_string());
    let healthz_bind = env::var("HEALTHZ_BIND").unwrap_or(BB_HEALTHZ_BIND.to_string());

    let _span = info_span!("{}/{}", endpoint, path).entered();

    let mut kkong = KingKong::new(&endpoint, &nats_addr, &healthz_bind).await;

    let _listener = kkong
        .new_future_kong(BB_PATH, |frame| async {
            info!("Received event");
            info!(?frame);
            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(frame)
        })
        .await;
    kkong.wait().await?;

    Ok(())
}
