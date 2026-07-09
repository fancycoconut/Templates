#[cfg(feature = "otlp")]
use opentelemetry::trace::TracerProvider as _;
#[cfg(feature = "otlp")]
use opentelemetry::{global, KeyValue};
#[cfg(feature = "otlp")]
use opentelemetry_otlp::{MetricExporter, SpanExporter};
#[cfg(feature = "otlp")]
use opentelemetry_sdk::{
    metrics::{PeriodicReader, SdkMeterProvider},
    runtime::Tokio,
    trace::TracerProvider,
    Resource,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[cfg(feature = "otlp")]
pub struct TelemetryGuard {
    tracer_provider: Option<TracerProvider>,
    meter_provider: Option<SdkMeterProvider>,
}

#[cfg(not(feature = "otlp"))]
pub struct TelemetryGuard;

#[cfg(feature = "otlp")]
impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        if let Some(tracer_provider) = &self.tracer_provider {
            if let Err(e) = tracer_provider.shutdown() {
                eprintln!("tracer shutdown error: {e}");
            }
        }
        if let Some(meter_provider) = &self.meter_provider {
            if let Err(e) = meter_provider.shutdown() {
                eprintln!("meter shutdown error: {e}");
            }
        }
    }
}

#[cfg(not(feature = "otlp"))]
pub fn init(log_level: &str) -> TelemetryGuard {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    TelemetryGuard
}

#[cfg(feature = "otlp")]
pub fn init(log_level: &str) -> TelemetryGuard {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));

    // OTLP export is opt-in: without an endpoint, the tonic exporter builder
    // fails to construct (empty/invalid URI), so skip it entirely rather than
    // let a missing/disabled endpoint crash the whole lambda.
    let otlp_enabled = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .map(|v| !v.is_empty())
        .unwrap_or(false);

    if !otlp_enabled {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer().json())
            .init();

        return TelemetryGuard {
            tracer_provider: None,
            meter_provider: None,
        };
    }

    let service_name = std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "{{project-name}}".into());

    let resource = Resource::new(vec![KeyValue::new("service.name", service_name)]);

    let tracer_provider = init_tracer(resource.clone());
    let meter_provider = init_meter(resource);

    global::set_tracer_provider(tracer_provider.clone());
    global::set_meter_provider(meter_provider.clone());

    let tracer = tracer_provider.tracer("{{project-name}}");

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    TelemetryGuard {
        tracer_provider: Some(tracer_provider),
        meter_provider: Some(meter_provider),
    }
}

#[cfg(feature = "otlp")]
fn init_tracer(resource: Resource) -> TracerProvider {
    let exporter = SpanExporter::builder()
        .with_tonic()
        .build()
        .expect("failed to create span exporter");

    TracerProvider::builder()
        .with_batch_exporter(exporter, Tokio)
        .with_resource(resource)
        .build()
}

#[cfg(feature = "otlp")]
fn init_meter(resource: Resource) -> SdkMeterProvider {
    let exporter = MetricExporter::builder()
        .with_tonic()
        .build()
        .expect("failed to create metric exporter");

    let reader = PeriodicReader::builder(exporter, Tokio).build();

    SdkMeterProvider::builder()
        .with_reader(reader)
        .with_resource(resource)
        .build()
}
