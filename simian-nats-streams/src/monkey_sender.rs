#[cfg(feature = "http-health")]
use simian_http_listener::MessageSender;
use crate::Monkey;
use anyhow::Error;
use simian_reactor::protocol::Frame;

/// MonkeySender wraps a Monkey to send Frame messages to NATS via HTTP
#[cfg(feature = "http-health")]
#[derive(Clone, Debug)]
pub struct MonkeySender {
    monkey: Monkey,
}

#[cfg(feature = "http-health")]
impl MonkeySender {
    pub fn new(monkey: Monkey) -> Self {
        Self { monkey }
    }
}

#[cfg(feature = "http-health")]
#[async_trait::async_trait]
impl MessageSender for MonkeySender {
    async fn send(&self, frame: Frame) -> Result<(), Error> {
        let bytes: bytes::Bytes = frame.into();
        self.monkey.msg(bytes).await?;
        Ok(())
    }
}
