# api-lambda

Rust AWS Lambda REST API template using axum + lambda_http with OpenTelemetry observability.

## Using This Template

### 1. Copy the template

```bash
# Clone the templates repo (if you haven't already)
git clone https://github.com/<your-org>/Templates.git

# Copy the template into a new project
cp -r Templates/rust/api-lambda ~/source/repos/my-new-api
cd ~/source/repos/my-new-api
```

### 2. Rename the project

Update the following with your project name (e.g. `orders-api`):

- **`Cargo.toml`** — change `name` and `[lib] name`
- **`serverless.yml`** — change `service`
- **`template.yaml`** — change `Description` and `OTEL_SERVICE_NAME`
- **`src/telemetry.rs`** — change the fallback service name in `unwrap_or_else`
- **`src/routes/health.rs`** — change the meter name in `global::meter(...)`

### 3. Initialise git and verify

```bash
git init
cargo test
cargo lambda watch  # confirm it runs locally
```

### 4. Set up CI/CD

Add these secrets to your GitHub repository (Settings > Secrets > Actions):
- `AWS_ACCESS_KEY_ID`
- `AWS_SECRET_ACCESS_KEY`

Push to `main` to trigger the deploy workflow.

## Prerequisites

- [Rust](https://rustup.rs/)
- [cargo-lambda](https://www.cargo-lambda.info/) — `pip3 install cargo-lambda` or `brew install cargo-lambda`
- [AWS CLI](https://aws.amazon.com/cli/) — configured with credentials
- [Serverless Framework v3](https://www.serverless.com/) — `npm install -g serverless`
- [Docker](https://www.docker.com/) — only required for SAM CLI local testing
- [AWS SAM CLI](https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/install-sam-cli.html) — optional, for integration testing

## Project Structure

```
src/
├── main.rs          # Lambda entrypoint — initialises telemetry, runs the router
├── lib.rs           # Library root — re-exports create_router for tests
├── router.rs        # Axum Router definition
├── telemetry.rs     # OpenTelemetry tracer + meter provider setup (OTLP)
└── routes/
    ├── mod.rs
    └── health.rs    # GET /health — example handler with tracing + metrics
tests/
└── api_test.rs      # Integration tests using tower::ServiceExt::oneshot
```

## Local Development

### Option A: cargo-lambda (recommended)

Fast iteration with hot-reload, no Docker required.

```bash
# Start the local Lambda emulator (watches for changes)
cargo lambda watch

# In another terminal, invoke the function
curl http://localhost:9000/health
```

### Option B: AWS SAM CLI

Closer to production — runs in a Docker container mimicking Lambda + API Gateway.

```bash
# Build the Lambda binary
cargo lambda build --release --arm64

# Start the local API Gateway
sam local start-api

# Test
curl http://127.0.0.1:3000/health
```

## Testing

```bash
cargo test
```

Tests use `tower::ServiceExt::oneshot` to send HTTP requests directly to the axum Router without starting the Lambda runtime or initialising OTEL — both degrade gracefully to no-ops when no provider is registered.

## Observability

Telemetry is configured via standard OpenTelemetry environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `OTEL_SERVICE_NAME` | Service name in traces/metrics | `api-lambda` |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | OTLP collector endpoint (gRPC) | — |
| `OTEL_RESOURCE_ATTRIBUTES` | Additional resource attributes | — |
| `RUST_LOG` | Log level filter | `info` |

Works with any OTEL-compatible backend: Honeycomb, Grafana Tempo, Jaeger, AWS X-Ray (via ADOT collector), Datadog, etc.

### Adding tracing to a handler

```rust
#[tracing::instrument]
pub async fn my_handler() -> impl IntoResponse {
    tracing::info!("processing request");
    // ...
}
```

The `#[instrument]` macro creates a span that is automatically exported to your OTEL backend.

## Deployment

```bash
# Build for Lambda
cargo lambda build --release --arm64

# Deploy to dev
serverless deploy

# Deploy to prod
serverless deploy --stage prod
```

Required GitHub Actions secrets for CI/CD:
- `AWS_ACCESS_KEY_ID`
- `AWS_SECRET_ACCESS_KEY`

## Adding New Routes

1. Create a handler in `src/routes/` (e.g. `src/routes/items.rs`)
2. Add `pub mod items;` to `src/routes/mod.rs`
3. Register the route in `src/router.rs`:
   ```rust
   Router::new()
       .route("/health", get(routes::health::handler))
       .route("/items", get(routes::items::list))
   ```
4. Add `#[tracing::instrument]` to the handler for automatic tracing
