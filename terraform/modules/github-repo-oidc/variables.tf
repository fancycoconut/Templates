variable "aws_region" {
  type        = string
  description = "AWS region the deploy role's HttpApi permissions are scoped to (should match the region lambdas-deploy.yml deploys into)."
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
