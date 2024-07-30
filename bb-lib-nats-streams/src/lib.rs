//! # rs-nats-streams
//!
//! Provides two kinds of communicators
//! 1. `Kong`
//! 2. `Monkey`
//!
//! Kongs are servers which setup a listener at a specific nats address ( e.g. "greet" )
//! The kong will listen on anything dotted after "greet" so this kong will respond to greet.sue
//!
//! Monkeys are for sending messages to kongs.  Monkey's can add a subject by adding a . after the
//! first subject name ( e.g. "greet.sue" )

mod core;
mod error;
mod frame_tools;
mod kong_tools;
mod util;
// mod kong_tools;

//TODO: Make me private?
pub mod towerish;

pub use anyhow::Error;
pub use error::NSLibError;

// pub use core::frame_handler::future::FrameFuture;
pub use frame_tools::{
    encoder::{Decoder, Encoder},
    frame::Frame,
    frame_handler::{FrameHandler, HandlesFrames},
    future::{FrameFuture, PinnedFuture},
    proc::Proc,
};
pub use kong_tools::KingKong;
pub use kong_tools::Kong;
pub use kong_tools::Monkey;

pub use towerish::NatsLayer;
pub use towerish::NatsSend;
pub use towerish::ResponseFuture;

use tower::BoxError;
use util::boxed_future_generator;
