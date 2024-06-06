use anyhow::Error;
use bb_lib_nats_streams::KingKong;
use tracing::{info, info_span};

const NATS_ADDR: &str = "nats://10.0.0.27:4222";

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _otel = bb_lib_tracing::initialize()?;
    let _span = info_span!("log-receiver").entered();
    // let _e = span.enter();
    let mut kkong = KingKong::new("gc-api", NATS_ADDR, "0.0.0.0:6668");
    let _listener = kkong
        .new_future_kong("log", |frame| async {
            info!("Received event");
            info!(?frame);
            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(frame)
        })
        .await;
    kkong.wait().await?;
    
    Ok(())
}
