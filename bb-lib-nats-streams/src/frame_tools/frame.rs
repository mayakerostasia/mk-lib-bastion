use super::encoder::{Decoder, Encoder};
use super::BBFrame;
use super::proc::Proc;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::convert::From;
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Frame {
    Ping,
    Pong,
    Msg(String),
    Bytes(Box<[u8]>),
    SendBox(SendBox),
    Json(Value),
    Close,
    Exec(Proc),
    Fin,
    Error(String),
}
impl BBFrame for Frame {}
impl Encoder for Frame {}
impl<'de> Decoder<'de, Frame> for Frame {}
impl From<Frame> for Bytes {
    fn from(value: Frame) -> Self {
        match value.encode() {
            Ok(frame) => frame.into(),
            Err(e) => Frame::Error(e.to_string()).into(),
        }
    }
}

impl From<Bytes> for Frame {
    fn from(value: Bytes) -> Self {
        match Frame::decode(&value) {
            Ok(frame) => frame,
            Err(e) => Frame::Error(e.to_string()),
        }
    }
}

impl Frame {
    pub fn json(json: Value) -> Frame {
        Frame::Json(json)
    }

    pub fn message(msg: &str) -> Frame {
        Frame::Msg(msg.to_string())
    }

    pub fn bytes(data: Box<[u8]>) -> Frame {
        Frame::Bytes(data)
    }

    pub fn exec(cmd: &str, args: Vec<&str>) -> Frame {
        Frame::Exec(Proc::new(cmd, args))
    }

    pub fn ping() -> Frame {
        Frame::Ping
    }

    pub fn pong() -> Frame {
        Frame::Pong
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SendBox {
    pub from: String,
    pub addr: String,
    pub data: Box<[u8]>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use tower::BoxError;
    use super::*;

    #[test]
    fn test_json_serialize() -> Result<(), BoxError> {
        let json = json!({"hello": "world"});
        let fram = Frame::json(json);
        let encoded = fram.encode()?;
        eprintln!("Encoded: {:?}", encoded);
        // assert!(true);
        Ok(())
    }
}
