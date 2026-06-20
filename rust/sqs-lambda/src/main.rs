use {{crate_name}}::{handler, settings::Settings, telemetry};

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    let settings = Settings::load().expect("failed to load configuration");
    let _guard = telemetry::init(&settings.logging.level);
    lambda_runtime::run(lambda_runtime::service_fn(handler::handle_event)).await
}
