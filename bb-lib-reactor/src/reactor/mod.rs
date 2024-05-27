pub use mako_battery::MakoBattery;
pub use mako_reactor::{ArcReactor, MakoReactor};

mod mako_battery;
mod mako_reactor;

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Error;
    use bb_lib_nats_streams::{Frame, Proc};

    async fn test_fn(frame: Frame) -> Result<Frame, Error> {
        Ok(Frame::pong())
    }

    #[tokio::test]
    async fn test_mako_reactor_service() -> Result<(), Error> {
        let mako = ArcReactor::new(120, 120);
        // mako.register_service("health", |frame| async { Ok(test_fn(frame).await?) } )
        //     .await?;

        // for _ in 0..10 {
        //     let resp = mako
        //         .call_service(Proc {
        //             cmd: "health".to_string(),
        //             args: vec![],
        //         })
        //         .await?;
        //     println!("Response is {:#?}", resp);
        // }

        // println!("Reactor: {:#?}", mako);
        // // assert_eq!(mako._battery.len(), 10);
        Ok(())
    }
}
