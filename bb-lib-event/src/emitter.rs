use anyhow::Error;
use bb_lib_nats_streams::{Frame, Monkey};
use tracing::instrument;
// use crate::event::{BBEvent, BBEventType};

const _NATS_ADDR: &str = "nats://10.2.4.106:4222";

#[derive(Debug, Clone, Default)]
pub struct Emitter;

impl Emitter {
    pub fn new() -> Emitter {
        Emitter
    }

    #[instrument(skip(self))]
    pub async fn emit_event(&self, event_name: &str, frame: Frame, nats_addr: &str) -> Result<(), Error> {
        // let nats_addr = std::env::var("NATS_ADDR").unwrap_or(NATS_ADDR.to_string());
        let monkey = Monkey::new("bastion-event.log", nats_addr).await;
        monkey.msg(frame).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_emitter() -> Result<(), Error> {
        // let kong = bb_lib_nats_streams::Kong::new("bastion-event", NATS_ADDR).await;
        // let _listener = kong.listen().await?;
        // let emitter = Emitter::new();
        // emitter.emit_event("test_event", Frame::ping()).await?;
        Ok(())
    }
}
