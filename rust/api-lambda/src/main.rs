use {{crate_name}}::{create_router, telemetry};

#[tokio::main]
async fn main() -> Result<(), lambda_http::Error> {
    let _guard = telemetry::init();
    lambda_http::run(create_router()).await
}
