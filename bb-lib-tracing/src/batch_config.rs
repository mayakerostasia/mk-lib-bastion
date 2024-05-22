use color_eyre::Result;
use core::convert::Into;
use core::option::Option;
use core::option::Option::{None, Some};
use core::result::Result::Ok;
use opentelemetry::sdk::trace::{BatchConfig, BatchSpanProcessor, Config, TraceRuntime};
use opentelemetry::trace::{TraceError, TracerProvider};
use opentelemetry::{global, sdk};
use opentelemetry_otlp::{Error, OtlpTracePipeline, SpanExporter, SpanExporterBuilder};

pub trait BatchConfigurable {
    fn install_batch_manual<B, R>(
        self,
        trace_config: Option<Config>,
        exporter_builder: Option<B>,
        batch_config: BatchConfig,
        runtime: R,
    ) -> Result<sdk::trace::Tracer, TraceError>
    where
        B: Into<SpanExporterBuilder>,
        R: TraceRuntime;
}

fn build_batch_with_config_and_exporter<R: TraceRuntime>(
    exporter: SpanExporter,
    batch_config: BatchConfig,
    trace_config: Option<Config>,
    runtime: R,
) -> sdk::trace::Tracer {
    let batch = BatchSpanProcessor::builder(exporter, runtime)
        .with_batch_config(batch_config)
        .build();
    let mut provider_builder = sdk::trace::TracerProvider::builder().with_span_processor(batch);
    if let Some(config) = trace_config {
        provider_builder = provider_builder.with_config(config);
    }
    let provider = provider_builder.build();
    let tracer =
        provider.versioned_tracer("opentelemetry-otlp", Some(env!("CARGO_PKG_VERSION")), None);
    let _ = global::set_tracer_provider(provider);
    tracer
}

impl BatchConfigurable for OtlpTracePipeline {
    fn install_batch_manual<B, R>(
        self,
        trace_config: Option<Config>,
        exporter_builder: Option<B>,
        batch_config: BatchConfig,
        runtime: R,
    ) -> Result<sdk::trace::Tracer, TraceError>
    where
        B: Into<SpanExporterBuilder>,
        R: TraceRuntime,
    {
        let exporter_builder = exporter_builder.map(|b| b.into());
        Ok(build_batch_with_config_and_exporter(
            exporter_builder
                .ok_or(Error::NoExporterBuilder)?
                .build_span_exporter()?,
            batch_config,
            trace_config,
            runtime,
        ))
    }
}