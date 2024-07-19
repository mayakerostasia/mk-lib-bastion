//! # Log Emitter
//! Listens to [BB_ENDPOINT].[BB_PATH] and Logs events sent to it
//! ## Env Vars:
//! BB_NATS_ADDR
//! BB_ENDPOINT
//! BB_PATH
//! BB_HEALTHZ_BIND
//!
//! ## Telemetry Vars
//! COLLECTOR_ENDPOINT
//! SERVICE_NAME
//! SERVICE_VERSION
//! ENV
//! RUST_LOG
use anyhow::Error;
use bb_lib_nats_streams::{KingKong, Frame};
use tracing::{info, info_span};
use std::{env, pin::Pin, future::Future};
use metrics::counter;

const BB_NATS_ADDR: &str = "nats://10.0.0.27:4222";
const BB_ENDPOINT: &str = "bb";
const BB_PATH: &str = "log";
const BB_HEALTHZ_BIND: &str = "0.0.0.0:4200";

fn count(name: &str) -> Result<(), Error> {
    counter!(name.to_string()).increment(1); 
    Ok(())
}

type BoxError = Box<dyn std::error::Error + Send + Sync>;

fn frame_handler(frame: Frame) -> Pin<Box<dyn Future<Output=Result<Frame, BoxError>> + Send + Sync>> {
    Box::pin( async move {
        let path = env::var("BB_PATH").unwrap_or(BB_PATH.to_string());
        let counter_name = format!("bb_nats_subject_{}", &path);
        count(&counter_name)?;
        info!("Received event");
        info!(?frame);
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(Frame::Fin)
    })
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    bb_lib_metrics::init_metrics().await?;
    let _otel = bb_lib_tracing::initialize()?;
    let nats_addr = env::var("BB_NATS_ADDR").unwrap_or(BB_NATS_ADDR.to_string());
    let endpoint= env::var("BB_ENDPOINT").unwrap_or(BB_ENDPOINT.to_string());
    let path = env::var("BB_PATH").unwrap_or(BB_PATH.to_string());
    let healthz_bind = env::var("HEALTHZ_BIND").unwrap_or(BB_HEALTHZ_BIND.to_string());

    let _span = info_span!("{}/{}", endpoint, path).entered();

    let mut kkong = KingKong::new(&endpoint, &nats_addr, &healthz_bind);

    let _listener = kkong
        .new_future_kong(&path.clone(), frame_handler)
        .await;
    kkong.wait().await?;

    Ok(())
}
