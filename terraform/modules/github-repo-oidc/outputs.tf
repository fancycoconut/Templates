output "deploy_role_arn" {
  description = "Store this as the AWS_DEPLOY_ROLE_ARN secret on the \"test\" GitHub Environment."
  value       = aws_iam_role.github_actions_deploy.arn
}
