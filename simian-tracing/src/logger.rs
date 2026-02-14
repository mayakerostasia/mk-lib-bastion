use anyhow::Error;
#[allow(unused_imports)]
use opentelemetry::logs::{LogError, LoggerProvider as _};
use opentelemetry_sdk::logs::LoggerProvider;
use std::process;
use url::Url;

use crate::{get_export_config, resource, ConfigType};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{logs::BatchConfigBuilder, runtime};

// fn _http_exporter(endpoint: String) -> HttpExporterBuilder {
//     opentelemetry_otlp::new_exporter()
//         .http()
//         // .with_protocol(Protocol::HttpBinary)
//         // .with_tls_config(ClientTlsConfig::default())
//         .with_export_config(get_export_config(endpoint, ConfigType::Logs))
// }

pub fn loki_logger(
    endpoint: String,
    environment: String,
    service_name: String,
    service_version: String,
) -> Result<(tracing_loki::Layer, tracing_loki::BackgroundTask), Error> {
    let (layer, task) = tracing_loki::builder()
        .label("environment", environment)?
        .label("service_name", service_name)?
        .label("service_version", service_version)?
        .extra_field("pid", format!("{}", process::id()))?
        .build_url(Url::parse(&endpoint).unwrap())?;
    Ok((layer, task))
}

pub fn _init_logger(endpoint: String) -> Result<LoggerProvider, LogError> {
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        // .with_tls_config(ClientTlsConfig::default())
        .with_protocol(opentelemetry_otlp::Protocol::Grpc)
        .with_export_config(get_export_config(endpoint, ConfigType::Logs));

    let log_provider = opentelemetry_otlp::new_pipeline()
        .logging()
        .with_resource(resource())
        .with_exporter(exporter)
        .with_batch_config(
            BatchConfigBuilder::default()
                .with_max_queue_size(8192)
                // .with_max_export_batch_size(8192)
                .build(),
        )
        .install_batch(runtime::Tokio)?;

    Ok(log_provider)
}
