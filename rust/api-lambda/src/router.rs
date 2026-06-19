use axum::{routing::get, Router};
use tower_http::trace::TraceLayer;

use crate::routes;

pub fn create_router() -> Router {
    Router::new()
        .route("/health", get(routes::health::handler))
        .layer(TraceLayer::new_for_http())
}
