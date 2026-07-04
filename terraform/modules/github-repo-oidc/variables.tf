variable "aws_region" {
  type        = string
  description = "AWS region the deploy role's HttpApi permissions are scoped to (should match the region lambdas-deploy.yml deploys into)."
}

variable "environment" {
  type        = string
  description = "SAM `Stage` parameter value used for this environment's deploy (see backend/lambdas/template.yaml). Functions/log groups are named with this prefix, e.g. `test-blanket-api`."
}

variable "github_org" {
  type    = string
  description = "Specify the GitHub organization or user the repo belongs to"
}

variable "github_repo" {
  type        = string
  description = "Specify the GitHub repo to create the OIDC for"
}

variable "github_environment" {
  type        = string
  description = "GitHub Environment name the role trusts. Must match the `environment:` key in lambdas-deploy.yml."
}

variable "create_oidc_provider" {
  type        = bool
  description = "Set to false if this AWS account already has a GitHub Actions OIDC provider registered for token.actions.githubusercontent.com."
}

variable "lambda_deployment_bucket_arn" {
  type        = string
  description = "ARN of the S3 bucket `sam deploy --s3-bucket` uploads Lambda artifacts to (see terraform/modules/s3)."
}
