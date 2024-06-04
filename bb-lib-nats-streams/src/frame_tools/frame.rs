use super::encoder::{Decoder, Encoder};
use super::BBFrame;
use super::proc::Proc;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::convert::From;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Frame {
    Ping,
    Pong,
    Msg(String),
    Bytes(Box<[u8]>),
    SendBox(SendBox),
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
        value.encode().into()
    }
}

impl From<Bytes> for Frame {
    fn from(value: Bytes) -> Self {
        Frame::decode(&value) 
    }
}

impl Frame {
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
