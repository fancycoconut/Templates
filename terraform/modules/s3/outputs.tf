output "lambda_deployments_bucket_arn" {
  description = "ARN of the S3 bucket used for SAM/Lambda deployment artifacts"
  value       = aws_s3_bucket.your_lambda_deployments.arn
}
