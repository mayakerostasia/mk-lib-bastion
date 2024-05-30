use anyhow::Error;
// use bb_lib_event::Emitter;
use bb_lib_nats_streams::{Frame, KingKong};
use tracing::{info, info_span};

const NATS_ADDR: &str = "nats://10.0.0.27:4222";

async fn process_frame(frame: Frame) -> Result<Frame, Error> {
    info!(?frame, "Received Frame");
    Ok(bb_lib_nats_streams::match_frame(frame).await?)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _otel = bb_lib_tracing::initialize()?;
    let _span = info_span!("root").entered();

    let mut kkong = KingKong::new("bastion-event", NATS_ADDR);
    let _listener = kkong
        .new_future_kong("log", |event| async { Ok(process_frame(event).await?) })
        .await?;

    kkong.wait().await?;
    Ok(())
}
