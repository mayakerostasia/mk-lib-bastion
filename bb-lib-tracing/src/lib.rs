pub use crate::telemetry::initialize;
pub use crate::telemetry::OtelGuard;

use opentelemetry::KeyValue;
use opentelemetry_otlp::{ExportConfig, Protocol};
use opentelemetry_sdk::resource::Resource;

pub mod prelude {
    pub use tracing::{
        debug, debug_span, error, error_span, info, info_span, instrument, span, trace, trace_span,
        warn, warn_span, Instrument, Level, Span,
    };
}
pub use prelude::*;

use opentelemetry_semantic_conventions::{
    resource::{DEPLOYMENT_ENVIRONMENT, SERVICE_NAME, SERVICE_VERSION},
    SCHEMA_URL,
};

use crate::tracer::init_tracer;

mod telemetry;
// mod sentry_layer;
mod tracer;
mod metrics;

#[allow(dead_code)]
enum ConfigType {
    Logs,
    Traces,
}
struct ServiceNames {
    service_name: String,
    service_version: String,
    deployment_environment: String,
}

impl ServiceNames {
    fn new() -> Self {
        Self {
            service_name: std::env::var("SERVICE_NAME").unwrap_or("default".into()),
            service_version: std::env::var("SERVICE_VERSION").unwrap_or("0.1.0".into()),
            deployment_environment: std::env::var("SERVICE_ENV").unwrap_or("dev".into()),
        }
    }

    fn out(&mut self) -> (String, String, String) {
        (
            self.service_name.to_string(),
            self.service_version.to_string(),
            self.deployment_environment.to_string(),
        )
    }
}
// Create a Resource that captures information about the entity for which telemetry is recorded.
fn resource() -> Resource {
    let (service_name, service_version, deployment_environment) = ServiceNames::new().out();
    Resource::from_schema_url(
        [
            KeyValue::new(SERVICE_NAME, service_name),
            KeyValue::new(SERVICE_VERSION, service_version),
            KeyValue::new(DEPLOYMENT_ENVIRONMENT, deployment_environment),
        ],
        SCHEMA_URL,
    )
}

fn get_export_config(endpoint: String, config_type: ConfigType) -> ExportConfig {
    match config_type {
        ConfigType::Logs => ExportConfig {
            endpoint,
            protocol: Protocol::HttpBinary,
            timeout: std::time::Duration::from_secs(3),
            
        },
        // ConfigType::Metrics => ExportConfig {
        //     endpoint: endpoint,
        //     protocol: Protocol::Grpc,
        //     timeout: std::time::Duration::from_secs(3),
        // },
        ConfigType::Traces => ExportConfig {
            endpoint,
            protocol: Protocol::Grpc,
            timeout: std::time::Duration::from_secs(3),
        },
        // _ => panic!("Invalid config type"),
    }
}
