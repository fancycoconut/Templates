use axum::{routing::get, Router};

use crate::routes;

pub fn create_router() -> Router {
    Router::new().route("/health", get(routes::health::handler))
}
