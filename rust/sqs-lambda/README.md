# AWS SQS Lambda Template

Rust AWS Lambda SQS consumer template using lambda_runtime + aws_lambda_events with OpenTelemetry observability.

## Using This Template

### 1. Install cargo-generate

```bash
cargo install cargo-generate
```

### 2. Generate a new project

```bash
# From a git repo
cargo generate --git https://github.com/<your-org>/Templates --path rust/sqs-lambda

# Or from a local clone
cargo generate --path /path/to/Templates/rust/sqs-lambda
```

You'll be prompted for a project name (e.g. `order-processor`). All files are automatically configured with your project name.

### 3. Verify

```bash
cd order-processor
cargo test
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

## Project Structure

```
config.toml              # Application configuration (log level, etc.)
src/
├── main.rs              # Lambda entrypoint — loads config, initialises telemetry, runs the handler
├── lib.rs               # Library root — re-exports modules for tests
├── handler.rs           # SQS event handler — processes message batches
├── settings.rs          # Configuration loading (config.toml + env var overrides)
└── telemetry.rs         # OpenTelemetry tracer + meter provider setup (OTLP)
tests/
└── handler_test.rs      # Integration tests using LambdaEvent construction
```

## Local Development

### Invoking with cargo-lambda

```bash
# Start the local Lambda emulator
cargo lambda watch

# In another terminal, invoke with a sample SQS event
cargo lambda invoke --data-file events/sqs-event.json
```

### Sample SQS event

Create `events/sqs-event.json` for local testing:

```json
{
  "Records": [
    {
      "messageId": "msg-001",
      "body": "{\"key\": \"value\"}",
      "eventSource": "aws:sqs",
      "eventSourceARN": "arn:aws:sqs:us-east-1:123456789012:my-queue"
    }
  ]
}
```

## Testing

```bash
cargo test
```

Tests construct `LambdaEvent<SqsEvent>` directly and call the handler function — no Lambda runtime or OTEL provider needed.

## Observability

### Logging

All log output is structured JSON written to stdout via `tracing-subscriber`. Lambda automatically captures stdout and forwards it to **CloudWatch Logs**.

#### Log level control

The log level is configured in `config.toml`:

```toml
[logging]
level = "info"
```

Override at runtime using environment variables with the `APP__` prefix:

```bash
APP__LOGGING__LEVEL=debug    # overrides config.toml
```

Or use the standard `RUST_LOG` env var (highest precedence):

```bash
RUST_LOG=debug
RUST_LOG=my_crate=debug,tower_http=trace
```

Precedence order: `RUST_LOG` > `APP__*` env vars > `config.toml`.

### OpenTelemetry (traces & metrics)

OTLP export is gated behind the `otlp` Cargo feature — it's off by default so `cargo check`/`cargo test`/CI don't pay to compile the tonic/prost gRPC stack. Build with it enabled when you want traces and metrics:

```bash
cargo lambda build --release --arm64 --features otlp
```

Without the `otlp` feature (or without `OTEL_EXPORTER_OTLP_ENDPOINT` set at runtime), the binary skips OTLP setup entirely and only does structured JSON logging to CloudWatch — no failed connection attempts, no extra deploy artifact size.

With the feature enabled, traces and metrics are exported via OTLP gRPC to a collector. For Lambda, add the [AWS Distro for OpenTelemetry (ADOT) Lambda layer](https://aws-otel.github.io/).

| Variable | Description | Default |
|----------|-------------|---------|
| `OTEL_SERVICE_NAME` | Service name in traces/metrics | `<project-name>` |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | OTLP collector endpoint (gRPC) | — |
| `OTEL_RESOURCE_ATTRIBUTES` | Additional resource attributes | — |
| `RUST_LOG` | Log level filter (overrides `config.toml`) | value from `config.toml` |

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

## Infrastructure

The SAM template (`template.yaml`) creates:

| Resource | Description |
|----------|-------------|
| **Queue** | Main SQS queue that triggers the Lambda |
| **DeadLetterQueue** | DLQ for messages that fail processing 3 times |
| **ConsumerFunction** | Lambda function triggered by the queue |

### Queue configuration

| Setting | Value | Notes |
|---------|-------|-------|
| Batch size | 10 | Max messages per Lambda invocation |
| Batching window | 5s | Wait up to 5s to fill a batch |
| Visibility timeout | 180s | 6x the function timeout (30s) |
| Max receive count | 3 | Messages move to DLQ after 3 failures |
| DLQ retention | 14 days | Time to inspect/replay failed messages |

### Sending test messages

```bash
# Get the queue URL from stack outputs
QUEUE_URL=$(aws cloudformation describe-stacks \
  --stack-name <stack-name> \
  --query 'Stacks[0].Outputs[?OutputKey==`QueueUrl`].OutputValue' \
  --output text)

# Send a message
aws sqs send-message --queue-url "$QUEUE_URL" --message-body '{"key": "value"}'
```

## Adding Message Processing Logic

Edit `src/handler.rs` and replace the `process_message` function:

```rust
async fn process_message(body: &str) -> Result<(), Error> {
    let payload: MyPayload = serde_json::from_str(body)?;
    // Your processing logic here
    Ok(())
}
```

For typed messages, define a struct and deserialize:

```rust
#[derive(Debug, Deserialize)]
struct OrderEvent {
    order_id: String,
    amount: f64,
}
```
