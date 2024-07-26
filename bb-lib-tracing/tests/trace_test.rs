use bb_lib_tracing::initialize;
use bb_lib_tracing::prelude::*;

#[tokio::test]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = initialize();
    // let subscriber = Registry::default().with(tracing_subscriber::fmt::layer());
    // tracing::subscriber::set_global_default(subscriber).unwrap();

    let _span = span!(Level::INFO, "test_span").entered();
    debug!("debug");
    info!("info");
    warn!("warn");
    error!("error");
    trace!("trace");
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    _span.exit();
    // drop(_guard);
    Ok(())
}
