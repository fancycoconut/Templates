use axum::Router;
use tower_http::trace::TraceLayer;

use crate::routing::RouteEntry;

pub fn create_router() -> Router {
    let mut router = Router::new();
    for entry in inventory::iter::<RouteEntry> {
        router = (entry.register)(router);
    }
    router.layer(TraceLayer::new_for_http())
}
