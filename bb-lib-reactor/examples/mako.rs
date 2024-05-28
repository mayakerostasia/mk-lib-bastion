use anyhow::Error;
use tokio::sync::Mutex;
use bb_lib_nats_streams::{Frame, KingKong, Proc, Monkey};
use bb_lib_reactor::ArcReactor;
use async_once::AsyncOnce;
use bb_lib_reactor::FramedFuture;
use lazy_static::lazy_static;
use serde_json::Value;
use tower::{Service, ServiceExt};

const BASE_URL: &str = "http://worldtimeapi.org/api/timezone";
const NATS_ADDR: &str = "nats://10.2.4.106:4222";

lazy_static! {
    static ref MONKEY_EXEC: AsyncOnce<Monkey> = AsyncOnce::new(async {
        Monkey::new("mako.exec",  NATS_ADDR).await
    });
    static ref KING_KONG: Mutex<KingKong> = Mutex::new(KingKong::new("mako", NATS_ADDR));
}

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

async fn call_tower_service(
    // srv: &mut impl Service<Frame, Response = Frame, Error = Error>,
    frame: Frame,
) -> Result<Frame, Error> {
    let mut kk = KING_KONG.lock().await;
    let fut = kk.call(frame).await;
    fut
}

fn tower_combinator(frame: Frame) -> FramedFuture<Frame> {
    Box::pin(call_tower_service(frame))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mako = ArcReactor::new(1, 1);
    mako.register_function("hi", call_time_future).await?;

    // let mut kkong = KingKong::new("mako", "nats://localhost:4222");
    let mut kkong = KING_KONG.lock().await;
    kkong.new_future_kong("mako", call_tower_service).await?;

    let mut _mako = mako.clone();

    let mut srv = tower::ServiceBuilder::new()
        .buffer(10)
        .concurrency_limit(5)
        .timeout(tokio::time::Duration::from_secs(1))
        .rate_limit(1, tokio::time::Duration::from_secs(1))
        .service(_mako);

    // for _ in 0..10 {
    //     let srv = srv.ready().await?;
    //     let fut = srv.call(Frame::exec(Proc {
    //         cmd: "hi".to_string(),
    //         args: vec!["America/New_York".to_string()],
    //     }));
    //     dbg!(fut.await?);
    // }
    kkong.wait().await?;
    Ok(())
}
