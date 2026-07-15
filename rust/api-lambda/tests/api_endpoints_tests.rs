use {{crate_name}}::routing::RouteEntry;

/// Snapshot of every route registered via `#[{{crate_name}}_macros::route(...)]`, collected
/// through `inventory` rather than read off `router.rs` — this is what's actually wired up,
/// not what a human remembers to keep in sync. If this test fails, the route surface
/// changed: review the diff and, if intentional, update `api-endpoints.verified.text`.
#[test]
fn registered_routes_match_snapshot() {
    let mut routes: Vec<(&str, &str)> = inventory::iter::<RouteEntry>
        .into_iter()
        .map(|entry| (entry.path, entry.method))
        .collect();
    routes.sort_unstable();

    let rendered = routes
        .into_iter()
        .map(|(path, method)| format!("{method} {path}"))
        .collect::<Vec<_>>()
        .join("\n");

    let verified_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/api-endpoints.verified.text");
    let expected = std::fs::read_to_string(verified_path).unwrap_or_default();

    assert_eq!(
        rendered.trim_end(),
        expected.trim_end(),
        "\nregistered routes changed — if intentional, update {verified_path}\n"
    );
}
