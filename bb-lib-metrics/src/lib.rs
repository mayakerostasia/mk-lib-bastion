use metrics_exporter_prometheus::PrometheusBuilder;
use std::net::SocketAddr;
use anyhow::Error;

pub async fn init_metrics() -> Result<(), Error> {
    PrometheusBuilder::new()
        // // For Push Gateway
        // .with_push_gateway(
        //  // "http://localhost:4200/api/v1/write",
        //     Duration::from_secs(1),
        //     None, None)?
        .with_http_listener(SocketAddr::new(
            "0.0.0.0".parse().expect("Couldn't Parse IP"),
            9010,
        ))
        .install()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use metrics::counter;

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
