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
mod kong;
mod kingkong;
mod monkey;
mod error;
mod util;

pub use anyhow::Error;
pub use error::NSLibError; 

pub use kong::Kong;
pub use kingkong::KingKong;
pub use monkey::Monkey;
pub use core::{
    encoder::{Decoder, Encoder}, 
    frames::Frame,
    // BastionRequest, BastionReply,
};

use util::boxed_future_generator;
pub use util::{PinnedFuture, BoxedFutureFn, annotate};
