use crate::core::encoder::{Decoder, Encoder};
use bytes::Bytes;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Proc {
    pub cmd: String,
    pub args: Vec<String>,
}

impl Proc {
    pub fn new(cmd: &str, args: Vec<&str>) -> Proc {
        Proc {
            cmd: cmd.to_string(),
            args: args.iter().map(|t| t.to_string()).collect(),
        }
    }
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
    Exec(Proc),
    Fin,
    Error(String),
}
impl Encoder for Frame {}
impl<'de> Decoder<'de, Frame> for Frame {}

impl Into<Bytes> for Frame {
    fn into(self) -> Bytes {
        self.encode().into()
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
