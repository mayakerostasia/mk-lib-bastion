pub use reactor::{ArcReactor, MakoBattery, MakoReactor};
use std::pin::Pin;
use std::future::Future;

mod reactor;

pub type Error = anyhow::Error;
pub type FramedFuture<O> = Pin<Box<dyn Future<Output = Result<O, Error>> + Send + Sync + 'static>>;
