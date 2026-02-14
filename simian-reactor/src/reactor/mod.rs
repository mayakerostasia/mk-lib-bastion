#[cfg(feature = "nats")]
pub use simian_nats_streams::{Frame, Proc};
pub use mako_battery::MakoBattery;
pub use mako_layer::MakoLayer;
pub use mako_reactor::MakoReactor;
use reactor_core::ReactorCore;
// pub use simian_nats_streams::{Decoder, Encoder};

mod mako_battery;
mod mako_layer;
mod mako_reactor;
mod mako_service;
mod reactor_core;

use tower::BoxError;

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use anyhow::Error;
//     use simian_nats_streams::{Frame, Proc};

//     async fn test_fn(frame: Frame) -> Result<Frame, Error> {
//         Ok(Frame::pong())
//     }

//     #[tokio::test]
//     async fn test_mako_reactor_service() -> Result<(), Error> {
//         let mako = MakoReactor::new(120, 120);
//         // mako.register_service("health", |frame| async { Ok(test_fn(frame).await?) } )
//         //     .await?;

//         // for _ in 0..10 {
//         //     let resp = mako
//         //         .call_service(Proc {
//         //             cmd: "health".to_string(),
//         //             args: vec![],
//         //         })
//         //         .await?;
//         //     println!("Response is {:#?}", resp);
//         // }

//         // println!("Reactor: {:#?}", mako);
//         // // assert_eq!(mako._battery.len(), 10);
//         Ok(())
//     }
// }
