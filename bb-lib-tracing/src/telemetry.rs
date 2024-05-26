use opentelemetry::logs::LogError;
use tracing_appender::non_blocking::WorkerGuard;

use crate::{get_export_config, resource, ConfigType};
use opentelemetry_otlp::{HttpExporterBuilder, WithExportConfig};

use opentelemetry_sdk::{trace::Tracer, logs::Logger, runtime};
use tracing_appender::rolling;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

use tonic::transport::channel::ClientTlsConfig;
// For Timer
use tracing_subscriber::fmt::time::ChronoLocal;

// use tracing::{error, warn};

use crate::init_tracer;

fn _http_exporter(endpoint: String) -> HttpExporterBuilder {
    let exporter = opentelemetry_otlp::new_exporter()
        .http()
        // .with_protocol(Protocol::HttpBinary)
        // .with_tls_config(ClientTlsConfig::default())
        .with_export_config(get_export_config(endpoint, ConfigType::Logs));
    exporter
}

fn init_logger(endpoint: String) -> anyhow::Result<Logger, LogError> {
    // // HTTP exporter
    // let exporter = http_exporter(endpoint);
    // // GRPC exporter
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_tls_config(ClientTlsConfig::default())
        .with_protocol(opentelemetry_otlp::Protocol::Grpc)
        .with_export_config(get_export_config(endpoint, ConfigType::Logs));

    opentelemetry_otlp::new_pipeline()
        .logging()
        .with_log_config(
            opentelemetry_sdk::logs::Config::default().with_resource(resource()), // .with_id_generator(RandomIdGenerator::default()),
        )
        .with_exporter(exporter)
        .install_batch(runtime::Tokio)
}

// Initialize tracing-subscriber and return OtelGuard for opentelemetry-related termination processing
fn mk_registry(endpoints: Endpoints) -> anyhow::Result<OtelGuard> {
    // let tracer = tracing::Subscriber
    let tracer = init_tracer(endpoints.tracer)?;
    let _trace_provider = tracer.provider().unwrap();


    // let meter_provider = init_meter_provider(endpoints.metrics)?;

    let logger = init_logger(endpoints.logger)?;
    let log_provider = logger.provider();
    let log_trace_bridge = OpenTelemetryTracingBridge::new(log_provider);

    // TODO: Logs directory should be configurable
    let debug_file = rolling::daily("./logs", "log.log");
    // Log warnings and errors to a separate file. Since we expect these events
    // to occur less frequently, roll that file on a daily basis instead.
    // TODO: Logs directory should be configurable
    // let warn_file = rolling::daily("./logs", "warnings").with_max_level(tracing::Level::WARN);
    
    // TODO: Logs directory should be configurable
    // let stderr = std::io::stderr();
    let all_files = debug_file;

    let (non_blocking, _guard) = tracing_appender::non_blocking(all_files);

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(OpenTelemetryLayer::new(tracer.clone()))
        // .with(log_trace_bridge)
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_file(true)
                .with_line_number(true)
                .with_timer(ChronoLocal::rfc_3339())
                .with_current_span(true)
                .with_target(true)
                .with_level(true)
                .with_writer(non_blocking)
                .and_then(log_trace_bridge)
        )
        .with(        
            tracing_subscriber::fmt::layer()
                .pretty()
                // .compact()
                .with_target(true)
                // .with_file(true)
                // .with_line_number(true)
                .with_timer(ChronoLocal::rfc_3339())
        )
        .init();

    let _guard = OtelGuard {
        writer_guard: _guard,
        tracer_provider: tracer,
        // meter_provider,
        logger_provider: logger,
    };

    Ok(_guard)
}

pub struct OtelGuard {
    pub writer_guard: WorkerGuard,
    pub tracer_provider: Tracer,
    // pub meter_provider: MeterProvider,
    pub logger_provider: Logger,
}

impl Drop for OtelGuard {
    fn drop(&mut self) {
        // if let Err(err) = self.meter_provider.shutdown() {
        //     eprintln!("{err:?}");
        // }
        // match self.meter_provider.shutdown() {
        //     Ok(_) => debug!("Meter provider shutdown successfully"),
        //     Err(err) => debug!("Meter provider already shutdown: {err:?}"),
        // };

        // let _ = opentelemetry::global::shutdown_meter_provider();

        let _shutdown_log = opentelemetry::global::shutdown_logger_provider();
        let _shutdown_trace = opentelemetry::global::shutdown_tracer_provider();
    }
}

struct Endpoints {
    tracer: String,
    logger: String,
    // metrics: String,
}

pub fn initialize() -> anyhow::Result<OtelGuard> {
    println!("Initializing telemetry");
    let collector_endpoint: String =
        std::env::var("COLLECTOR_ENDPOINT").unwrap_or("http://localhost:4317".to_string());
    let logs_endpoint: String =
        std::env::var("LOGGER_ENDPOINT").unwrap_or(collector_endpoint.clone());
    let tracer_endpoint: String =
        std::env::var("TRACER_ENDPOINT").unwrap_or(collector_endpoint.clone());
    // let _metrics_endpoint: String =
    //     std::env::var("METRICS_ENDPOINT").unwrap_or(collector_endpoint.clone());

    let endpoints = Endpoints {
        logger: logs_endpoint,
        tracer: tracer_endpoint,
        // metrics: metrics_endpoint,
    };

    let _tel = mk_registry(endpoints)?;
    Ok(_tel)
}
