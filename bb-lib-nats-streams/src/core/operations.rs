

// use crate::Frame::SendBox;
// use crate::Frame::

use crate::core::frames::{SendBox, Proc};
use crate::Decoder;
use tracing::info;

use crate::Error;
use super::frames::Frame;

// #[instrument]
pub async fn match_frame<'de>(frame: Frame) -> Result<Frame, Error> {
    // let cmd = Frame::decode(frame);
    match frame {
        // Frame::Register(registration) => got_register(registration),
        Frame::SendBox(SendBox { from, addr, data }) => got_sendbox(from, addr, data),
        Frame::Msg(msg) => got_msg(msg),
        Frame::Exec(proc) => got_exec(proc),
        Frame::Bytes(data) => got_bytes(data),
        Frame::Ping => got_ping(),
        Frame::Pong => got_pong(),
        Frame::Close => got_close(),
    }
}

pub fn got_ping() -> Result<Frame, Error> {
    info!("Got ping");
    Ok(Frame::pong())
}

pub fn got_pong() -> Result<Frame, Error> {
    info!("Got pong");
    Ok(Frame::pong())
}

pub fn got_msg(msg: String) -> Result<Frame, Error> {
    info!("Got message: {}", msg);
    Ok(Frame::pong())
}

pub fn got_bytes(data: Box<[u8]>) -> Result<Frame, Error> {
    info!("Got bytes: {:?}", data);
    Ok(Frame::pong())
}

pub fn got_sendbox(from: String, addr: String, data: Box<[u8]>) -> Result<Frame, Error> {
    info!(
        "Got sendbox: from: {}, addr: {}, data: {:?}",
        from, addr, data
    );
    Ok(Frame::pong())
}

// pub fn got_register(reg: MonkeyRegistration) -> () {
//     info!("Got registration request");
//     info!("ListenAddr {:?}", reg.socket_addr);
//     info!("MonkeyAddress {:?}", reg.monkey_address);
//     info!("CommandSet {:?}", reg.command_set);
// }

pub fn got_close() -> Result<Frame, Error> {
    info!("Got close");
    Ok(Frame::pong())
}

pub fn got_exec(proc: Proc) -> Result<Frame, Error> {
    info!("Got exec: {:?}", proc);
    Ok(Frame::pong())
}
