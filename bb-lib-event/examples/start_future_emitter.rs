use anyhow::Error;

const _NATS_ADDR: &str = "nats://10.0.0.27:4222";

// async fn process_frame(frame: Frame) -> Result<Frame, Error> {
//     info!(?frame, "Received Frame");
//     Ok(bb_lib_nats_streams::match_frame(frame).await?)
// }

#[tokio::main]
async fn main() -> Result<(), Error> {
    // let _otel = bb_lib_tracing::initialize()?;
    // let span = info_span!("root").entered();
    // // let _e = span.enter();
    // let mut kkong = KingKong::new("bastion-event", NATS_ADDR);
    // let _listener = kkong
    //     .new_future_kong("test", |frame| async move { process_frame(frame).await })
    //     .await?;
    // // let listener = kong.listen().await?;
    // let emitter = Emitter::new();
    // let _ = emitter
    //     .emit_event(
    //         "test-event",
    //         Frame::message(format!("Hello from {}", kkong.name).as_str()),
    //     )
    //     .await?;
    // span.exit();
    // tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    Ok(())
}
