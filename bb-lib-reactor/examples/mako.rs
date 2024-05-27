use bb_lib_reactor::FramedFuture;
use anyhow::Error;
use bb_lib_reactor::ArcReactor;
use bb_lib_nats_streams::{Frame, Proc};
use tower::{Service, ServiceExt};
use serde_json::Value;

const BASE_URL: &str = "http://worldtimeapi.org/api/timezone";

async fn call_time(args: Vec<String>) -> Result<Frame, Error> {
    assert_eq!(args.len(), 1);
    let endpoint = &args[0];

    println!("Endpoint is : {}", endpoint.as_str());
    let request = reqwest::get(format!("{}/{}", BASE_URL, endpoint)).await?;

    println!("Status: {}", request.status());
    let val: Value = request.json().await?;

    Ok(Frame::message(serde_json::to_string(&val)?))
}

fn call_time_future(args: Vec<String>) -> FramedFuture<Frame> {
    Box::pin(call_time(args))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mako = ArcReactor::new(1, 1);
    mako.register_function("hi", call_time_future).await?;

    let mut _mako = mako.clone();

    let mut srv = tower::ServiceBuilder::new()
        .buffer(10)
        .concurrency_limit(5)
        .timeout(tokio::time::Duration::from_secs(1))
        .rate_limit(1, tokio::time::Duration::from_secs(1))
        .service(_mako);

    for _ in 0..10 {
        let srv = srv.ready().await?;
        let fut = srv.call(Frame::exec(Proc { cmd: "hi".to_string(), args: vec!["America/New_York".to_string()] }));
        dbg!(fut.await?);
    }
    Ok(())
}
