use anyhow::Error;
use simian_nats_streams::{Decoder, Frame, Monkey};
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _otel = simian_tracing::initialize();

    // Get the NATS Address
    let nats_addr = std::env::var("NATS_ADDR").unwrap_or("nats://10.2.4.106:4222".to_string());

    // Other Process
    // Initialize a Monkey to send the request
    let monkey = Monkey::new("time.new_york", nats_addr.as_str()).await;
    let resp = monkey.msg(Frame::exec("America/New_York", vec![])).await?;

    let frame = Frame::decode(&resp.payload);
    match frame {
        Ok(Frame::Msg(val)) => {
            let resul: Value = serde_json::from_str(&val)?;
            serde_json::to_writer_pretty(std::io::stdout(), &resul)?
        }
        _ => unimplemented!("Not Allowed!"),
    };
    Ok(())
}
