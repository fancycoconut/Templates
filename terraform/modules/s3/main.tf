
resource "aws_s3_bucket" "your_lambda_deployments" {
  bucket = "${var.environment}-your-lambda-deployments"
}