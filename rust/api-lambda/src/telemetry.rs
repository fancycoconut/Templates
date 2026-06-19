use opentelemetry::trace::TracerProvider as _;
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::{MetricExporter, SpanExporter};
use opentelemetry_sdk::{
    metrics::{PeriodicReader, SdkMeterProvider},
    runtime::Tokio,
    trace::TracerProvider,
    Resource,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub struct TelemetryGuard {
    tracer_provider: TracerProvider,
    meter_provider: SdkMeterProvider,
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        if let Err(e) = self.tracer_provider.shutdown() {
            eprintln!("tracer shutdown error: {e}");
        }
        if let Err(e) = self.meter_provider.shutdown() {
            eprintln!("meter shutdown error: {e}");
        }
    }
}

pub fn init() -> TelemetryGuard {
    let service_name =
        std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "{{project-name}}".into());

    let resource = Resource::new(vec![KeyValue::new("service.name", service_name)]);

    let tracer_provider = init_tracer(resource.clone());
    let meter_provider = init_meter(resource);

    global::set_tracer_provider(tracer_provider.clone());
    global::set_meter_provider(meter_provider.clone());

    let tracer = tracer_provider.tracer("{{project-name}}");

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    TelemetryGuard {
        tracer_provider,
        meter_provider,
    }
}

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
