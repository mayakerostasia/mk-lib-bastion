use anyhow::anyhow;

use crate::{util::PinnedFuture, Decoder, Error, Frame, KingKong, Monkey, PinnedFuture};

mod kong_conf;

pub struct Barrel {}

// pub struct Banana {
//     subject: String,
//     monkey: Monkey,
// }
//
pub type FnPointer<T> = fn(T) -> PinnedFuture<T>;

use std::collections::HashMap;

pub struct Banana<T> {
    endpoints: HashMap<String, FnPointer<T>>
}

pub trait KongBananaBag<T> {
    fn open(&self) -> HashMap<String, FnPointer<T>>;
    fn register_funk(&mut self, name: &str, funk: FnPointer<T>) -> Result<(), Error>;
}

pub trait BananaExt<T> {
    fn peel(&self, name: &str) -> Option<&FnPointer<T>>;
    fn eat(&self, name: &str, args: Vec<T>) -> PinnedFuture<T>;
}

impl BananaExt<Frame> for Banana<Frame> {
    fn peel(&self, name: &str) -> Option<&FnPointer<Frame>> {
        self.endpoints.get(name)
    }
    fn eat(&self, name: &str, args: Frame) -> PinnedFuture<Frame> {
        if let Some(funk) = self.peel(name) {
            Box::pin(funk(args))
        } else {
            async { Err(anyhow!("Not allowed!"))}
        }
    }
}

// impl Banana {
//     async fn shoot(&self, frame: Frame) ->Result<Frame, Error> {
//         Ok(Frame::decode(&self.monkey.msg(frame).await?.payload))
//     }
// }

// pub trait KongHolder {};

pub trait DefaultKongExt 
{
    async fn default_kongs(&mut self) ->Result<(), Error> ;
}

impl DefaultKongExt for KingKong {
    async fn default_kongs(&mut self) ->Result<(), Error> {
        self.new_kong("ping", || async { Frame::pong() }).await?;
        self.new_kong("endpoints", || async { Frame::message("ping, endpoints".to_string()) }).await?;
        // self.new_future_kong("exec", |frame| drive_frame_to_proc(frame)).await?;
        Ok(())
    }
}

pub struct _KongEndpoints {
    inner: Vec<String>,
}

pub trait KongEndpoints {
    fn register_endpoint(&mut self) -> Result<(), Error>;
    fn list_endpoints(&mut self) -> Result<(), Error>;
}
