use anyhow::Error;
use simian_nats_streams::{Decoder, Encoder};
use bytes::Bytes;
use core::fmt::Formatter;
use serde::{Deserialize, Serialize};
use tracing::error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimianEventType {
    Info(String),
}
impl Encoder for SimianEventType {}
impl Decoder<'_, SimianEventType> for SimianEventType {}
impl From<SimianEventType> for Bytes {
    fn from(value: SimianEventType) -> Self {
        match value.encode() {
            Ok(val) => val.into(),
            Err(e) => {
                error!("Whoops! Had an error {e}");
                panic!("exiting");
            }
        }
    }
}
impl std::fmt::Display for SimianEventType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            SimianEventType::Info(s) => write!(f, "{}", s),
        }?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimianEvent {
    name: String,
    _type: SimianEventType,
}
impl Encoder for SimianEvent {}
impl Decoder<'_, SimianEvent> for SimianEvent {}
impl From<SimianEvent> for Bytes {
    fn from(value: SimianEvent) -> Self {
        match value.encode() {
            Ok(val) => val.into(),
            Err(e) => {
                error!("Whoops! Had an error {e}");
                panic!("exiting");
            }
        }
    }
}

impl SimianEvent {
    pub fn new(name: &str, event_type: SimianEventType) -> Self {
        SimianEvent {
            name: name.to_string(),
            _type: event_type,
        }
    }

    pub fn emit(&self) -> Result<(), Error> {
        Ok(())
    }
}

impl std::fmt::Display for SimianEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "event={} type={}", self.name, self._type)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bb_event() {
        let event = SimianEvent::new("test", SimianEventType::Info("test".to_string()));
        assert_eq!(event.to_string(), "event=test type=test");
    }
}
