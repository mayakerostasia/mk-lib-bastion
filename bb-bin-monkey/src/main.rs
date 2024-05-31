// use tower::BoxError;
use anyhow::Error;
use clap::Parser;
use bb_lib_nats_streams::{Monkey, Frame};
const DEFAULT_NATS_ADDR: &str = "nats://10.2.4.106:4222";
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

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = MonkeyCli::parse();
    // Monkey Call
    let monkey = Monkey::new(&args.subject, &args.nats_addr).await;
    let resp = monkey.msg_timeout(
        Frame::exec(&args.cmd, args.args.iter().map(|a| a.as_str()).collect()),
        None
    ).await?;
    println!("{:#?}", resp);
    Ok(())
}
