//! # Monkey Sender
//!
use anyhow::{anyhow, Error};
use bb_lib_nats_streams::{Decoder, Frame, Monkey};
use clap::Parser;
use std::{io::Write, time::Duration};
const DEFAULT_NATS_ADDR: &str = "nats://10.2.4.106:4222";
use tracing::debug_span;

/// A nats message tool using the bb-frame-protocol
///
/// Example:
/// ```no-run
/// bb-bin-monkey --subject "gc-api.exec" --cmd "_all"
/// ```
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct MonkeyCli {
    /// nats subject to target
    #[arg(short, long)]
    subject: String,

    /// Frame::Proc.cmd -> The command string you'd like to send
    #[arg(short, long)]
    cmd: Option<String>,

    /// Frame::Proc.args -> The args you'd like to include in the proc object
    #[arg(short, long)]
    args: Vec<String>,

    /// The nats server address listening for the message
    #[arg(short,long, default_value = DEFAULT_NATS_ADDR)]
    nats_addr: String,
}

pub async fn handle_ret_frame(frame: bytes::Bytes) -> Result<Frame, Error> {
    let fram = Frame::decode(&frame).map_err(|e| anyhow!(e))?;
    match fram.clone() {
        Frame::Msg(stri) => {
            eprintln!("Ret frame = Msg");
            writeln!(std::io::stdout(), "Message -> {:#?}", stri)?;
        }
        Frame::Json(val) => {
            eprintln!("Ret frame = Json");
            serde_json::to_writer_pretty(std::io::stdout(), &val)?;
        }
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
    let resp = match &args.cmd {
        Some(cmd) => {
            monkey
                .msg_timeout(
                    Frame::exec(cmd, args.args.iter().map(|a| a.as_str()).collect()),
                    None,
                )
                .await?
        }
        None => {
            monkey
                .msg_timeout(Frame::ping(), Some(Duration::from_millis(500)))
                .await?
        }
    };
    println!("Payload -> {:#?}", &resp.payload);
    handle_ret_frame(resp.payload).await?;
    Ok(())
}
