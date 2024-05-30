use anyhow::Error;
use bb_lib_event::Emitter;
use bb_lib_nats_streams::{Frame, KingKong};
use tracing::{info, info_span};

const NATS_ADDR: &str = "nats://10.0.0.27:4222";

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _otel = bb_lib_tracing::initialize()?;
    let span = info_span!("root").entered();
    // let _e = span.enter();
    let mut kkong = KingKong::new("bastion-event", NATS_ADDR);
    let _listener = kkong
        .new_kong("test", || async {
            info!("Received event");
            "Hi"
        })
        .await?;
    // let listener = kong.listen().await?;
    let emitter = Emitter::new();

    let _ = emitter
        .emit_event("test-event", Frame::Msg("Emitted".to_string()))
        .await?;
    span.exit();
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    Ok(())
}
