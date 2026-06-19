use {{crate_name}}::{create_router, settings::Settings, telemetry};

#[tokio::main]
async fn main() -> Result<(), lambda_http::Error> {
    let settings = Settings::load().expect("failed to load configuration");
    let _guard = telemetry::init(&settings.logging.level);
    lambda_http::run(create_router()).await
}
