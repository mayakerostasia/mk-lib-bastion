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
use async_once::AsyncOnce;
use bb_lib_nats_streams::{Frame, KingKong};
use config::Config;
use lazy_static::lazy_static;
use metrics::counter;
use serde::Deserialize;
use std::{future::Future, pin::Pin};
use tracing::{info, info_span};

lazy_static! {
    static ref CONF: AsyncOnce<EventEmitterSettings> =
        AsyncOnce::new(async { config().await.expect("Couldn't Parse Config File") });
}

#[derive(Clone, Debug, Deserialize)]
pub struct EventEmitterSettings {
    pub nats_addr: String,
    pub kong_bind_addr: String,
    pub king_kong_subject: String,
    pub kong_subject: String,
}

async fn config() -> Result<EventEmitterSettings, Error> {
    let source =
        config::File::with_name(&std::env::var("BB_CONFIG").expect("Please Set BB_CONFIG ENV var"));
    let conf = Config::builder().add_source(source);
    let conf = conf.build()?;
    Ok(conf.try_deserialize::<EventEmitterSettings>()?)
}
fn count(name: &str) -> Result<(), Error> {
    counter!(name.to_string()).increment(1);
    Ok(())
}

type BoxError = Box<dyn std::error::Error + Send + Sync>;

fn frame_handler(
    frame: Frame,
) -> Pin<Box<dyn Future<Output = Result<Frame, BoxError>> + Send + Sync>> {
    Box::pin(async move {
        let conf = CONF.get().await;
        let endpoint = conf.kong_subject.clone();
        let path = conf.king_kong_subject.clone();
        let _span = info_span!("{}/{}", endpoint, path).entered();
        let counter_name = format!("bb_nats_subject_{}_{}", &endpoint, &path);
        count(&counter_name)?;
        info!(?frame, "Received event");
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(frame)
    })
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Init o11y
    bb_lib_metrics::init_metrics().await?;
    let _otel = bb_lib_tracing::initialize()?;

    //  Get Configureation
    let conf = CONF.get().await;

    let mut kkong = KingKong::new(
        &conf.king_kong_subject,
        &conf.nats_addr,
        &conf.kong_bind_addr,
    )
    .await;
    let _listener = kkong
        .new_future_kong(&conf.kong_subject, frame_handler)
        .await;
    kkong.wait().await?;

    Ok(())
}
