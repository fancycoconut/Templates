use aws_lambda_events::sqs::SqsEvent;
use lambda_runtime::{Error, LambdaEvent};
use opentelemetry::{global, KeyValue};

#[tracing::instrument(skip(event), fields(record_count))]
pub async fn handle_event(event: LambdaEvent<SqsEvent>) -> Result<(), Error> {
    let (sqs_event, _context) = event.into_parts();
    let records = sqs_event.records;

    tracing::Span::current().record("record_count", records.len());
    tracing::info!(count = records.len(), "processing SQS batch");

    let meter = global::meter("{{project-name}}");
    let counter = meter.u64_counter("sqs.messages.received").build();

    for record in &records {
        let message_id = record.message_id.as_deref().unwrap_or("unknown");
        let body = record.body.as_deref().unwrap_or("");

        tracing::info!(message_id, "processing message");
        counter.add(1, &[KeyValue::new("status", "processed")]);

        process_message(body).await?;
    }

    tracing::info!(count = records.len(), "batch complete");
    Ok(())
}

async fn process_message(body: &str) -> Result<(), Error> {
    tracing::debug!(body, "message body");
    // TODO: Add your message processing logic here
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_lambda_events::sqs::{SqsEvent, SqsMessage};
    use lambda_runtime::{Context, LambdaEvent};

    fn build_event(bodies: &[&str]) -> LambdaEvent<SqsEvent> {
        let records: Vec<SqsMessage> = bodies
            .iter()
            .enumerate()
            .map(|(i, body)| SqsMessage {
                message_id: Some(format!("msg-{i}")),
                body: Some(body.to_string()),
                ..Default::default()
            })
            .collect();

        let sqs_event = SqsEvent { records };
        LambdaEvent::new(sqs_event, Context::default())
    }

    #[tokio::test]
    async fn handles_single_message() {
        let event = build_event(&[r#"{"key": "value"}"#]);
        let result = handle_event(event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn handles_multiple_messages() {
        let event = build_event(&["message-1", "message-2", "message-3"]);
        let result = handle_event(event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn handles_empty_batch() {
        let event = build_event(&[]);
        let result = handle_event(event).await;
        assert!(result.is_ok());
    }
}
