use axum::Json;
use opentelemetry::{global, KeyValue};

#[tracing::instrument]
pub async fn handler() -> Json<serde_json::Value> {
    let meter = global::meter("api-lambda");
    let counter = meter.u64_counter("health_check.requests").build();
    counter.add(1, &[KeyValue::new("status", "ok")]);

    Json(serde_json::json!({ "status": "ok" }))
}
