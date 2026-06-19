use axum::Json;
use opentelemetry::{global, KeyValue};

#[tracing::instrument]
pub async fn handler() -> Json<serde_json::Value> {
    tracing::info!("Health check requested");

    let meter = global::meter("{{project-name}}");
    let counter = meter.u64_counter("health_check.requests").build();
    counter.add(1, &[KeyValue::new("status", "ok")]);

    Json(serde_json::json!({ "status": "ok" }))
}
