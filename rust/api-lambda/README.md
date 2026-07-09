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

`.github/workflows/` ships two alternative deploy pipelines — each is self-contained (check, build, deploy). **Delete whichever one you're not using**, otherwise every push runs both and attempts two deployments.

- **`deploy-using-iam-role.yml`** (recommended) — authenticates to AWS via GitHub OIDC, no long-lived credentials stored in GitHub. Requires:
  - An AWS IAM role trusting GitHub's OIDC provider, exposed as secret `AWS_DEPLOY_ROLE_ARN`
  - Repository variable `AWS_REGION`
  - Update the `--s3-bucket` placeholder in the workflow's `sam deploy` step to your deployment bucket
- **`deploy.yml`** — authenticates with a static AWS access key. Requires secrets `AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY`, plus repository variable `AWS_REGION`.

Both jobs build once on a native `ubuntu-24.04-arm` runner (matching the Lambda's `arm64` architecture) and pass the SAM build artifact to `deploy`, so the binary that's tested is the one that's shipped.

Push to `main` to trigger the deploy workflow.

## Prerequisites

- [Rust](https://rustup.rs/)
- [cargo-lambda](https://www.cargo-lambda.info/) — `pip3 install cargo-lambda` or `brew install cargo-lambda`
- [AWS CLI](https://aws.amazon.com/cli/) — configured with credentials
- [AWS SAM CLI](https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/install-sam-cli.html)
- [Docker](https://www.docker.com/) — only required for SAM CLI local testing

## Project Structure

```
config.toml              # Application configuration (log level, etc.)
src/
├── main.rs              # Lambda entrypoint — loads config, initialises telemetry, runs the router
├── lib.rs               # Library root — re-exports create_router for tests
├── router.rs            # Axum Router definition with request logging middleware
├── settings.rs          # Configuration loading (config.toml + env var overrides)
├── telemetry.rs         # OpenTelemetry tracer + meter provider setup (OTLP)
└── routes/
    ├── mod.rs
    └── health.rs        # GET /health — example handler with tracing + metrics
tests/
└── api_test.rs          # Integration tests using tower::ServiceExt::oneshot
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

### Logging

All log output is structured JSON written to stdout via `tracing-subscriber`. Lambda automatically captures stdout and forwards it to **CloudWatch Logs** — no extra configuration needed. A log group is created at `/aws/lambda/<function-name>`.

Request/response logging is handled by `tower-http`'s `TraceLayer`, which logs every incoming request (method, URI) and outgoing response (status code, latency) at the `DEBUG` level.

#### Log level control

The log level is configured in `config.toml`:

```toml
[logging]
level = "info"
```

You can override this at runtime using environment variables with the `APP__` prefix (double underscore as section separator):

```bash
APP__LOGGING__LEVEL=debug    # overrides config.toml
```

Or bypass the config file entirely with the standard `RUST_LOG` env var, which takes highest precedence:

```bash
RUST_LOG=debug                                  # includes request/response logs from TraceLayer
RUST_LOG=my_crate=debug,tower_http=trace        # fine-grained per-crate control
```

Precedence order: `RUST_LOG` > `APP__*` env vars > `config.toml`.

#### Adding logging to a handler

```rust
#[tracing::instrument]
pub async fn my_handler() -> impl IntoResponse {
    tracing::info!("processing request");
    // ...
}
```

The `#[instrument]` macro creates a span around the handler. Use `tracing::info!`, `tracing::warn!`, `tracing::error!`, etc. inside for structured log events.

### OpenTelemetry (traces & metrics)

OTLP export is gated behind the `otlp` Cargo feature — it's off by default so `cargo check`/`cargo test`/CI don't pay to compile the tonic/prost gRPC stack. Build with it enabled when you want traces and metrics:

```bash
cargo lambda build --release --arm64 --features otlp
```

Without the `otlp` feature (or without `OTEL_EXPORTER_OTLP_ENDPOINT` set at runtime), the binary skips OTLP setup entirely and only does structured JSON logging to CloudWatch — no failed connection attempts, no extra deploy artifact size.

With the feature enabled, traces and metrics are exported via OTLP gRPC to a collector. For this to work in Lambda, add the [AWS Distro for OpenTelemetry (ADOT) Lambda layer](https://aws-otel.github.io/), which runs a collector as a Lambda extension.

Configuration uses standard OpenTelemetry environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `OTEL_SERVICE_NAME` | Service name in traces/metrics | `<project-name>` |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | OTLP collector endpoint (gRPC) | — |
| `OTEL_RESOURCE_ATTRIBUTES` | Additional resource attributes | — |
| `RUST_LOG` | Log level filter (overrides `config.toml`) | value from `config.toml` |

Works with any OTEL-compatible backend: Honeycomb, Grafana Tempo, Jaeger, AWS X-Ray (via ADOT collector), Datadog, etc.

## Deployment

```bash
# Build for Lambda (add --features otlp to enable OTLP tracing/metrics)
cargo lambda build --release --arm64

# First deploy (interactive — sets up samconfig.toml)
sam deploy --guided

# Subsequent deploys
sam deploy
```

See [Set up CI/CD](#4-set-up-cicd) for the GitHub Actions secrets/variables each deploy workflow needs.

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
