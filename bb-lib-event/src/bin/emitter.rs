use anyhow::Error;
use bb_lib_event::Emitter;
use bb_lib_nats_streams::Frame;
use tracing::info_span;

const NATS_ADDR: &str = "nats://10.0.0.27:4222";

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _otel = bb_lib_tracing::initialize()?;
    let _span = info_span!("root").entered();
    let emitter = Emitter::new();
    for _ in [..10] {
        emitter
            .emit_event(
                "bastion-event.log",
                Frame::Msg("Emitted".to_string()),
                NATS_ADDR,
            )
            .await?;
    }
    Ok(())
}
