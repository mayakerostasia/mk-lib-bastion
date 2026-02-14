use simian_nats_streams::{Frame, KingKong, Monkey};
use simian_reactor::{FrameFuture, MakoLayer};
use reqwest::Identity;
use serde_json::Value;
use std::sync::Arc;
use std::{future::Future, pin::Pin};
use tokio::sync::Mutex;
use tower::{BoxError, Service, ServiceExt};

const BASE_URL: &str = "http://worldtimeapi.org/api/timezone";
const NATS_ADDR: &str = "nats://10.2.4.106:4222";

pub type Error = tower::BoxError;

async fn call_time(args: Vec<String>) -> Result<Frame, Error> {
    assert_eq!(args.len(), 1);
    let endpoint = &args[0];

    println!("Endpoint is : {}", endpoint.as_str());
    let request = reqwest::get(format!("{}/{}", BASE_URL, endpoint)).await?;

    println!("Status: {}", request.status());
    let val: Value = request.json().await?;

    Ok(Frame::message(serde_json::to_string(&val)?.as_str()))
}

async fn frame_handler(frame: Frame) -> Result<Frame, Error> {
    match frame {
        Frame::Msg(msg) => Ok(Frame::message(msg.as_str())),
        Frame::Ping => Ok(Frame::pong()),
        Frame::Exec(proc) => Ok(call_time(proc.args).await?),
        _ => unimplemented!("Not Allowed!"),
    }
}

#[derive(Debug)]
struct FrameHandler;

impl Service<Frame> for FrameHandler {
    type Response = Frame;
    type Future = FrameFuture<
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + Sync>>,
        Self::Response,
        Self::Error,
    >;
    type Error = BoxError;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Frame) -> Self::Future {
        FrameFuture::new(Box::pin(async { Ok(frame_handler(req).await?) }))
    }
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let srv = tower::ServiceBuilder::new()
        .buffer(10)
        .concurrency_limit(5)
        .timeout(tokio::time::Duration::from_secs(1))
        .rate_limit(1, tokio::time::Duration::from_secs(1))
        // .layer(MakoLayer::new(1, 1))
        .service(FrameHandler);

    let reactor_service = Arc::new(Mutex::new(srv));

    let mut kkong = KingKong::new("time", NATS_ADDR);
    kkong
        .new_tower_kong("time", reactor_service.clone())
        .await?;

    // Monkey Call
    let monkey = Monkey::new("time.time", NATS_ADDR).await;
    let frame: Frame = Frame::exec("time", vec!["America/New_York"]);
    let byt: bytes::Bytes = frame.into();
    let resp = monkey.msg(byt).await?;

    println!("Response: {:?}", resp);
    // Ok(())

    // Manual Call
    // for _ in 0..1 {
    //     let _srv = reactor_service.clone();
    //     let mut srv = _srv.lock().await;
    //     let reacto = srv.ready().await?;
    //     let fut: Frame = reacto
    //         .call(Frame::exec("time", vec!["America/New_York"]))
    //         .await
    //         .unwrap();
    //     dbg!(fut);
    // }
    // kkong.wait().await?;
    Ok(())
}
