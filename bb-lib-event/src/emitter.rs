use anyhow::Error;
use bb_lib_nats_streams::{Frame, Monkey};
use tracing::instrument;
// use crate::event::{BBEvent, BBEventType};

const NATS_ADDR: &str = "nats://10.0.0.27:4222";

#[derive(Debug, Clone)]
pub struct Emitter;

impl Emitter {
    pub fn new() -> Emitter {
        Emitter
    }

    #[instrument(skip(self))]
    pub async fn emit_event(&self, event_name: &str, frame: Frame) -> Result<(), Error> {
        let nats_addr = std::env::var("NATS_ADDR").unwrap_or(NATS_ADDR.to_string());
        let monkey = Monkey::new("bastion-event.log", &nats_addr).await;
        monkey.msg(frame).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_emitter() -> Result<(), Error> {
        let kong = bb_lib_nats_streams::Kong::new("bastion-event", NATS_ADDR).await;
        let _listener = kong.listen().await?;
        let emitter = Emitter::new();
        emitter.emit_event("test_event", Frame::ping()).await?;
        Ok(())
    }
}
