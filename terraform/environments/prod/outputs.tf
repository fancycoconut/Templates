output "deploy_role_arn" {
  description = "Store this as the AWS_DEPLOY_ROLE_ARN secret on the \"test\" GitHub Environment."
  value       = module.somerepo_github_repo_oidc.deploy_role_arn
}
