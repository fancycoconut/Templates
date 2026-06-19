# AWS API Lambda Template

Rust AWS Lambda REST API template using axum + lambda_http with OpenTelemetry observability.

## Using This Template

### 1. Install cargo-generate

```bash
cargo install cargo-generate
```

### 2. Generate a new project

```bash
# From a git repo
cargo generate --git https://github.com/<your-org>/Templates --path rust/api-lambda

# Or from a local clone
cargo generate --path /path/to/Templates/rust/api-lambda
```

You'll be prompted for a project name (e.g. `orders-api`). All files are automatically configured with your project name.

### 3. Verify

```bash
cd orders-api
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
- [AWS SAM CLI](https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/install-sam-cli.html)
- [Docker](https://www.docker.com/) — only required for SAM CLI local testing

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

# First deploy (interactive — sets up samconfig.toml)
sam deploy --guided

# Subsequent deploys
sam deploy
```

Required GitHub Actions secrets for CI/CD:
- `AWS_ACCESS_KEY_ID`
- `AWS_SECRET_ACCESS_KEY`

## Authentication (OIDC)

If you enabled OIDC during project generation, the API Gateway is configured with a JWT authorizer. All requests must include a valid Bearer token in the `Authorization` header.

```bash
curl -H "Authorization: Bearer <token>" https://<api-id>.execute-api.<region>.amazonaws.com/health
```

The authorizer validates tokens against your OIDC provider's issuer URL and audience. To change these after generation, edit the `JwtConfiguration` in `template.yaml`.

### Supported providers

| Provider | Issuer URL |
|----------|------------|
| Entra ID (Azure AD) | `https://login.microsoftonline.com/{tenant-id}/v2.0` |
| Auth0 | `https://{your-domain}.auth0.com/` |
| Cognito | `https://cognito-idp.{region}.amazonaws.com/{user-pool-id}` |
| Okta | `https://{your-domain}.okta.com/oauth2/default` |
| Google | `https://accounts.google.com` |

### Excluding routes from auth

To make specific routes public (e.g. health check), set `Auth: NONE` on the event in `template.yaml`:

```yaml
Events:
  HealthPath:
    Type: HttpApi
    Properties:
      Path: /health
      Method: GET
      Auth:
        Authorizer: NONE
```

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
