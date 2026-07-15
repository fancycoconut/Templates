use axum::Router;

/// One entry per `#[{{crate_name}}_macros::route(METHOD, "path")]`-annotated handler,
/// collected at compile time via `inventory` — `router.rs` folds every entry into the
/// app's `Router` instead of listing routes by hand. Never construct this directly;
/// it's built by the `route` attribute macro.
pub struct RouteEntry {
    pub method: &'static str,
    pub path: &'static str,
    pub register: fn(Router) -> Router,
}

inventory::collect!(RouteEntry);
