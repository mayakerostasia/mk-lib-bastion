use anyhow::Error;
use bb_lib_event::Emitter;
use bb_lib_nats_streams::{KingKong, Frame};
use tracing::{info, info_span};

const NATS_ADDR: &str = "nats://10.0.0.27:4222";

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _otel = bb_lib_tracing::initialize()?;
    let span = info_span!("root").entered();
    let emitter = Emitter::new();
    for _ in [..10] {
        let _event = emitter.emit_event("bastion-event.log", Frame::Msg("Emitted".to_string())).await?;
    };
    Ok(())
}
