use crate::core::encoder::{Decoder, Encoder};
use bytes::Bytes;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Proc {
    pub cmd: String,
    pub args: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SendBox {
    pub from: String,
    pub addr: String,
    pub data: Box<[u8]>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Frame {
    Ping,
    Pong,
    Msg(String),
    Bytes(Box<[u8]>),
    SendBox(SendBox),
    Close,
    // Register(MonkeyRegistration),
    Exec(Proc),
}
impl Encoder for Frame {}
impl<'de> Decoder<'de, Frame> for Frame {}

impl Into<Bytes> for Frame {
    fn into(self) -> Bytes {
        self.encode().into()
    }
}

impl Frame {
    pub fn message(msg: String) -> Frame {
        Frame::Msg(msg)
    }

    pub fn bytes(data: Box<[u8]>) -> Frame {
        Frame::Bytes(data)
    }

    pub fn exec(proc: Proc) -> Frame {
        Frame::Exec(proc)
    }

    pub fn ping() -> Frame {
        Frame::Ping
    }

    pub fn pong() -> Frame {
        Frame::Pong
    }
}
