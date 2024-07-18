use std::future::Future;
use std::pin::Pin;
use std::{net::SocketAddr, time::Duration};

use anyhow::Error;
use metrics::counter;
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusRecorder};

pub async fn init_metrics() -> Result<
    (),
    // //  Return Recoreder/Future
    // (
    //     PrometheusRecorder,
    //     Pin<Box<dyn Future<Output = Result<(), hyper::Error>> + Send>>,
    // ),
    Error,
> {
    PrometheusBuilder::new()
        // // For Push Gateway
        // .with_push_gateway(
        //  // "http://localhost:4200/api/v1/write",
        //     Duration::from_secs(1),
        //     None, None)?
        .with_http_listener(SocketAddr::new(
            "0.0.0.0".parse().expect("Couldn't Parse IP"),
            9010,
        )).install()?;
    
    // //  Return Recoreder/Future
    // let builder = PrometheusBuilder::new();
    // builder.install().expect("Failed to install recorder/exporter");
    // let (recorder, exporter) = builder.build().expect("Failed to build Recorder/Exporter");
    // Ok((recorder, exporter))
    Ok(())
}

#[cfg(test)]
mod tests{
    fn add_counter() -> String {
        let counter = counter!("counter.nicotest", "service" => "http");
        counter.increment(1);
        "Counted".to_string()
    }

    #[tokio::test]
    async fn test_metrics() -> Result<(), Error> {
        init_metrics().await?;

        println!("Hello, world!");

        // for _ in 0..10 {
        // dbg!(add_counter());
        // }

        // exporter.await?;
        let mut counter = 0;
        while counter < 10 {
            eprintln!("Sleeping");
            dbg!(add_counter());
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            counter += 1
        }
        Ok(())
    }

}
