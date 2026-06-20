# Templates

A mono repo of project templates for various languages and tech stacks. Each template is scaffolded with CI/CD, observability, and best practices baked in — ready to generate a new project via [cargo-generate](https://github.com/cargo-generate/cargo-generate) or equivalent tooling.

## Templates

### Rust

| Template | Description | Path |
|----------|-------------|------|
| [API Lambda](rust/api-lambda/) | AWS Lambda REST API using axum + lambda_http with OpenTelemetry, SAM deployment, and optional OIDC auth | `rust/api-lambda` |
| [SQS Lambda](rust/sqs-lambda/) | AWS Lambda SQS consumer using lambda_runtime + aws_lambda_events with OpenTelemetry and DLQ support | `rust/sqs-lambda` |

## Usage

Templates use `cargo-generate` for scaffolding. Install it once:

```bash
cargo install cargo-generate
```

Then generate a new project from any template:

```bash
# From this repo
cargo generate --git https://github.com/fancycoconut/Templates --path <template-path>

# Example
cargo generate --git https://github.com/fancycoconut/Templates --path rust/api-lambda
```

See each template's own README for full setup instructions, prerequisites, and local development guides.

## Repo Structure

```
rust/
├── api-lambda/     # REST API Lambda template (axum + API Gateway)
└── sqs-lambda/     # SQS consumer Lambda template
```
