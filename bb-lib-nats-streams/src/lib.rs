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
mod kingkong;
mod kong;
mod monkey;
mod util;

pub use anyhow::Error;
pub use error::NSLibError;

pub use core::{
    encoder::{Decoder, Encoder},
    frames::{Frame, Proc},
    match_frame,
    // BastionRequest, BastionReply,
};
pub use kingkong::KingKong;
pub use kong::Kong;
pub use monkey::Monkey;

use std::future::Future;
use std::pin::Pin;
use util::boxed_future_generator;
pub use util::{annotate, BoxedFutureFn};

pub type PinnedFuture<O> = Pin<Box<dyn Future<Output = Result<O, Error>> + Send + Sync + 'static>>;
