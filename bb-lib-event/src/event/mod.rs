use anyhow::Error;
use bb_lib_nats_streams::{Decoder, Encoder};
use bytes::Bytes;
use tracing::error;
use core::fmt::Formatter;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BBEventType {
    Info(String),
}
impl Encoder for BBEventType {}
impl Decoder<'_, BBEventType> for BBEventType {}
impl From<BBEventType> for Bytes { 
    fn from(value: BBEventType) -> Self {
        match value.encode() {
            Ok(val) => val.into(),
            Err(e) => {error!("Whoops! Had an error {e}"); panic!("exiting");},
        }
    }
}
impl std::fmt::Display for BBEventType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            BBEventType::Info(s) => write!(f, "{}", s),
        }?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BBEvent {
    name: String,
    _type: BBEventType,
}
impl Encoder for BBEvent {}
impl Decoder<'_, BBEvent> for BBEvent {}
impl From<BBEvent> for Bytes { 
    fn from(value: BBEvent) -> Self {
        match value.encode() {
            Ok(val) => val.into(),
            Err(e) => {error!("Whoops! Had an error {e}"); panic!("exiting");},
        }
    }
}

impl BBEvent {
    pub fn new(name: &str, event_type: BBEventType) -> Self {
        BBEvent {
            name: name.to_string(),
            _type: event_type,
        }
    }

    pub fn emit(&self) -> Result<(), Error> {
        Ok(())
    }
}

impl std::fmt::Display for BBEvent {
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
        let event = BBEvent::new("test", BBEventType::Info("test".to_string()));
        assert_eq!(event.to_string(), "event=test type=test");
    }
}
