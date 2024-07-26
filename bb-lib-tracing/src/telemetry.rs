use opentelemetry_sdk::logs::LoggerProvider;
#[allow(unused_imports)]
use opentelemetry::logs::{LogError, LoggerProvider as _};
use opentelemetry::trace::TracerProvider;
#[allow(unused_imports)]
use opentelemetry::trace::{Tracer, TraceError, TracerProvider as _};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
// use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::fmt::time::ChronoLocal;
use tracing::debug;
use crate::{init_tracer, init_logger, loki_logger};


// Initialize tracing-subscriber and return OtelGuard for opentelemetry-related termination processing
fn mk_registry(endpoints: Endpoints) -> anyhow::Result<OtelGuard> {
    // let tracer = tracing::Subscriber
    let tracer_provider = init_tracer(endpoints.tracer)?;
    let tracer = tracer_provider.tracer("bb-trace");
    let (loki_layer, log_task) = loki_logger(endpoints.loki.clone())?;
    // let log_layer_provider = init_logger(dbg!(endpoints.logger))?
    // let log_layer = OpenTelemetryTracingBridge::new(&log_layer_provider);
    // let otel_trace_layer = OpenTelemetryLayer::new(tracer);
    let otel_trace_layer = tracing_opentelemetry::layer()
        .with_tracer(tracer);
    
    // // TODO: Logs directory should be configurable
    // let debug_file = rolling::daily("./logs", "log.log");
    // let all_files = debug_file;
    // let (_non_blocking, _guard) = tracing_appender::non_blocking(all_files);

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(otel_trace_layer)
        // .with(log_layer)
        .with(loki_layer)
        .with(
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_target(true)
                .with_timer(ChronoLocal::rfc_3339()),
        )
        .init();

    let log_handle = tokio::spawn(log_task);

    let _guard = OtelGuard {
        // writer_guard: _guard,
        log_handle,
        // tracer_provider: tracer,
        // meter_provider,
        // logger_provider: log_layer_provider,
    };

    Ok(_guard)
}

pub struct OtelGuard {
    // pub writer_guard: WorkerGuard,
    pub log_handle: tokio::task::JoinHandle<()>,
    // pub tracer_provider: TracerProvider,
    // pub meter_provider: MeterProvider,
    // pub logger_provider: LoggerProvider,
}

impl Drop for OtelGuard {
    fn drop(&mut self) {
        // opentelemetry::global::shutdown_logger_provider();
        // self.log_handle.
        // self.log_handle.abort();
        // opentelemetry::global::shutdown_tracer_provider();
    }
}

struct Endpoints {
    tracer: String,
    logger: String,
    loki: String,
    // metrics: String,
}

pub fn initialize() -> anyhow::Result<OtelGuard> {
    debug!("Initializing telemetry");
    let collector_endpoint: String =
        std::env::var("COLLECTOR_ENDPOINT").unwrap_or("http://otel:4317".to_string());
    let logs_endpoint: String =
        std::env::var("LOGGER_ENDPOINT").unwrap_or("http://otel:4317".to_string());
    let loki_endpoint: String =
        std::env::var("LOKI_ENDPOINT").unwrap_or("http://loki:3100".to_string());
    // let tracer_endpoint: String =
    //     std::env::var("TRACER_ENDPOINT").unwrap_or(collector_endpoint.clone());
    // let _metrics_endpoint: String =
    //     std::env::var("METRICS_ENDPOINT").unwrap_or(collector_endpoint.clone());

    let endpoints = Endpoints {
        logger: logs_endpoint,
        tracer: collector_endpoint,
        loki: loki_endpoint,
        // metrics: metrics_endpoint,
    };

    let _tel = mk_registry(endpoints)?;
    Ok(_tel)
}
