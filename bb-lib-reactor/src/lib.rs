pub use future::FrameFuture;
pub use reactor::{MakoBattery, MakoLayer, MakoReactor};
pub type Error = anyhow::Error;
pub use error::ReactorError;

mod error;
mod future;
mod reactor;

mod proc_service;

// pub type PinnedFuture<O> = Pin<Box<dyn Future<Output = Result<O, Error>> + Send + Sync + 'static>>;
