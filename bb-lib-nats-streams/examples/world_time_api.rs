use anyhow::Error;
use bb_lib_nats_streams::{Decoder, Frame, KingKong, Monkey, Proc};
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;

const BASE_URL: &str = "http://worldtimeapi.org/api/timezone";

async fn call_time(frame: Frame) -> Result<Frame, Error> {
    // Match the frame to get the endpoint
    let proc = process_frame(frame)?;
    let endpoint = proc.cmd;
    println!("Endpoint is : {}", endpoint.as_str());
    let request = reqwest::get(format!("{}/{}", BASE_URL, endpoint)).await?;
    println!("Status: {}", request.status());
    let val: Value = request.json().await?;
    Ok(Frame::message(serde_json::to_string(&val)?))
}

 fn process_frame(frame: Frame) -> Result<Proc, Error> {
    match frame {
        Frame::Exec(proc) => Ok(proc),
        _ => unimplemented!(),
    }
}

fn call_time_future(frame: Frame) -> FramedFuture<Frame> {
    Box::pin(call_time(frame))
}

type FramedFuture<O> = Pin<Box<dyn Future<Output = Result<O, Error>> + Send + 'static>>;

#[allow(non_snake_case)]
#[tokio::main]
async fn main() -> Result<(), Error> {
    let _otel = bb_lib_tracing::initialize();

    // Get the NATS Address
    let nats_addr = std::env::var("NATS_ADDR").unwrap_or("nats://10.2.4.106:4222".to_string());

    // Initialize the KingKong
    let mut kkong = KingKong::new("time", nats_addr.as_str());
    // Register the service
    kkong.new_future_kong( "new_york", call_time_future ).await?;

    // Other Process
    // Initialize a Monkey to send the request
    let monkey = Monkey::new("time.new_york", nats_addr.as_str()).await;
    let resp = monkey
        .msg(Frame::exec(Proc {
            cmd: "America/New_York".to_string(),
            args: vec![],
        }))
        .await?;

    let frame = Frame::decode(&resp.payload);
    match frame {
        Frame::Msg(val) => { 
            let resul: Value = serde_json::from_str(&val)?;
            serde_json::to_writer_pretty(std::io::stdout(), &resul)? 
        },
        _ => unimplemented!("Not Allowed!")
    }
    // tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    Ok(())
}
