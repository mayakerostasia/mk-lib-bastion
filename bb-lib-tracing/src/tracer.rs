use opentelemetry_sdk::trace::TracerProvider;
#[allow(unused_imports)]
use opentelemetry::trace::{Tracer, TraceError, TracerProvider as _};
use crate::{get_export_config, resource, ConfigType};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::BatchConfigBuilder;
use opentelemetry_sdk::{
    runtime,
    trace::{RandomIdGenerator, Sampler, SpanLimits},
};
// use tonic::transport::channel::ClientTlsConfig;
// use sentry::Client;

// fn http_exporter(endpoint: String) -> HttpExporterBuilder {
//     let exporter = opentelemetry_otlp::new_exporter()
//         .http()
//         // .with_tls_config(ClientTlsConfig::default())
//         .with_export_config(get_export_config(endpoint, ConfigType::Traces));
//     exporter
// }
//

// Construct Tracer for OpenTelemetryLayer
pub fn init_tracer(endpoint: String) -> anyhow::Result<TracerProvider, TraceError> {
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        // .with_tls_config(ClientTlsConfig::default())
        .with_protocol(opentelemetry_otlp::Protocol::Grpc)
        .with_export_config(get_export_config(endpoint, ConfigType::Traces));

    let provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_trace_config(
            opentelemetry_sdk::trace::Config::default()
                .with_resource(resource())
                .with_sampler(Sampler::AlwaysOn)
                .with_id_generator(RandomIdGenerator::default()) // .with_resource(resource())
                .with_span_limits(SpanLimits::default()),
        )
        .with_batch_config(BatchConfigBuilder::default()
            .with_max_queue_size(8192)
            // .with_max_export_batch_size(8192)
            .build()
        )
        .with_exporter(exporter)
        .install_batch(runtime::Tokio)?;
    
    Ok(provider)
}
