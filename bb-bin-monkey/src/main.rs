use std::io::Write;
use anyhow::{anyhow, Error};
use bb_lib_nats_streams::{Frame, Monkey, Decoder};
use clap::Parser;
const DEFAULT_NATS_ADDR: &str = "nats://10.2.4.106:4222";
use tracing::debug_span;
/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct MonkeyCli {
    /// Name of the person to greet
    #[arg(short, long)]
    subject: String,

    /// Number of times to greet
    #[arg(short, long)]
    cmd: String,

    #[arg(short, long)]
    args: Vec<String>,

    #[arg(short,long, default_value = DEFAULT_NATS_ADDR)]
    nats_addr: String,
}

pub async fn handle_ret_frame(frame: bytes::Bytes) -> Result<Frame, Error> {
    let fram = Frame::decode(&frame).map_err(|e| anyhow!(e))?;
    match fram.clone() {
        Frame::Msg(stri) => {
            eprintln!("Ret frame = Msg");
            writeln!(std::io::stdout(), "Message -> {:#?}", stri)?;
        },
        Frame::Json(val) => {
            eprintln!("Ret frame = Json");
            serde_json::to_writer_pretty(std::io::stdout(), &val)?;
        },
        _ => {
            eprintln!("Ret frame = ? ");
            eprintln!("Other -> {:#?}", fram);
        }
    }
    Ok(fram)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _otel = bb_lib_tracing::initialize()?;
    let _span = debug_span!("MonkeySpan").entered();
    let args = MonkeyCli::parse();
    // Monkey Call
    let monkey = Monkey::new(&args.subject, &args.nats_addr).await;
    let resp = monkey
        .msg_timeout(
            Frame::exec(&args.cmd, args.args.iter().map(|a| a.as_str()).collect()),
            None,
        )
        .await?;
    println!("Payload -> {:#?}", &resp.payload);
    handle_ret_frame(resp.payload).await?;
    Ok(())
}
