use anyhow::anyhow;
use bb_lib_nats_streams::{Frame, KingKong, Proc};
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;

pub type Error = tower::BoxError;

const BASE_URL: &str = "http://worldtimeapi.org/api/timezone";

async fn call_time(frame: Frame) -> Result<Frame, Error> {
    // Match the frame to get the endpoint
    let proc = process_frame(frame)?;
    let endpoint = proc.cmd;
    println!("Endpoint is : {}", endpoint.as_str());
    let request = reqwest::get(format!("{}/{}", BASE_URL, endpoint)).await?;
    println!("Status: {}", request.status());
    let val: Value = request.json().await?;
    Ok(Frame::message(serde_json::to_string(&val)?.as_str()))
}

fn process_frame(frame: Frame) -> Result<Proc, Error> {
    match frame {
        Frame::Exec(proc) => Ok(proc),
        _ => Err(anyhow!("Failing Frame").into()),
    }
}

fn call_time_future(frame: Frame) -> FramedFuture<Frame> {
    Box::pin(call_time(frame))
}

type FramedFuture<O> = Pin<Box<dyn Future<Output = Result<O, Error>> + Send>>;

#[allow(non_snake_case)]
#[tokio::main]
async fn main() -> Result<(), Error> {
    let _otel = bb_lib_tracing::initialize();

    // Get the NATS Address
    let nats_addr = std::env::var("NATS_ADDR").unwrap_or("nats://10.2.4.106:4222".to_string());

    // Initialize the KingKong
    let mut kkong = KingKong::new("time", nats_addr.as_str(), "0.0.0.0:6662").await;
    // Register the service
    kkong.new_future_kong("new_york", call_time_future).await?;

    // Other Process
    // Initialize a Monkey to send the request
    // let monkey = Monkey::new("time.new_york", nats_addr.as_str()).await;
    // debug!("Monkey is {:#?}", monkey);
    // let resp = monkey.msg(Frame::exec("America/New_York", vec![])).await?;

    // let frame = Frame::decode(&resp.payload);
    // match frame {
    //     Ok(Frame::Msg(val)) => {
    //         let resul: Value = serde_json::from_str(&val)?;
    //         serde_json::to_writer_pretty(std::io::stdout(), &resul)?
    //     }
    //     _ => unimplemented!("Not Allowed!"),
    // }

    // Kong Survive
    let kong_handle = tokio::task::spawn(async move { Ok::<_, Error>(kkong.wait().await?) });
    let _ = tokio::join!(kong_handle);
    Ok(())
}
