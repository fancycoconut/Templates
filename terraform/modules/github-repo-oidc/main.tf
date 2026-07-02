# One-time AWS-side setup for keyless GitHub Actions deploys. Creates (or
# reuses) the GitHub OIDC identity provider and an IAM role that
# lambdas-deploy.yml assumes via sts:AssumeRoleWithWebIdentity — no AWS
# access key/secret is ever stored in GitHub.
#
# This is NOT part of the SAM app (../../template.yaml) and is not applied
# by any CI workflow. Apply it yourself, once, with credentials you already
# have configured locally.

data "aws_caller_identity" "current" {}

# There can only be one OIDC provider per account for a given URL. Leave
# create_oidc_provider = true the first time this is applied anywhere in
# the account; set it to false for any later reuse (e.g. a second repo).
resource "aws_iam_openid_connect_provider" "github" {
  count = var.create_oidc_provider ? 1 : 0

  url            = "https://token.actions.githubusercontent.com"
  client_id_list = ["sts.amazonaws.com"]

  # AWS validates GitHub's OIDC tokens against its own managed trust store
  # for this well-known issuer and no longer actually checks the
  # thumbprint, but the argument is still required by the provider.
  thumbprint_list = ["6938fd4d98bab03faadb97b34396831e3780aea1"]
}

locals {
  oidc_provider_arn = var.create_oidc_provider ? aws_iam_openid_connect_provider.github[0].arn : "arn:aws:iam::${data.aws_caller_identity.current.account_id}:oidc-provider/token.actions.githubusercontent.com"
}

data "aws_iam_policy_document" "github_actions_assume_role" {
  statement {
    effect  = "Allow"
    actions = ["sts:AssumeRoleWithWebIdentity"]

    principals {
      type        = "Federated"
      identifiers = [local.oidc_provider_arn]
    }

    condition {
      test     = "StringEquals"
      variable = "token.actions.githubusercontent.com:aud"
      values   = ["sts.amazonaws.com"]
    }

    # Restricts to workflow runs deploying through the "test" GitHub
    # Environment on this repo — must match the `environment:` key in
    # lambdas-deploy.yml. Forks and other branches/environments are rejected.
    condition {
      test     = "StringLike"
      variable = "token.actions.githubusercontent.com:sub"
      values   = ["repo:${var.github_org}/${var.github_repo}:environment:${var.github_environment}"]
    }
  }
}

resource "aws_iam_role" "github_actions_deploy" {
  name               = "github-actions-${var.github_repo}-deploy"
  assume_role_policy = data.aws_iam_policy_document.github_actions_assume_role.json
}

# Scoped to what `sam build && sam deploy` needs for template.yaml (stack
# "baby-tracker", functions/roles prefixed "dev-"/"baby-tracker-"). Treat
# this as a starting point: SAM/CloudFormation deploys touch a lot of
# surface area, so expect to add an action or two the first time this runs
# if CloudFormation reports AccessDenied.
data "aws_iam_policy_document" "sam_deploy" {
  statement {
    sid    = "CloudFormation"
    effect = "Allow"
    actions = [
      "cloudformation:CreateChangeSet",
      "cloudformation:DescribeChangeSet",
      "cloudformation:ExecuteChangeSet",
      "cloudformation:DeleteChangeSet",
      "cloudformation:DescribeStacks",
      "cloudformation:DescribeStackEvents",
      "cloudformation:DescribeStackResource",
      "cloudformation:DescribeStackResources",
      "cloudformation:GetTemplateSummary",
      "cloudformation:ListStackResources",
      "cloudformation:CreateStack",
      "cloudformation:UpdateStack",
      "cloudformation:ValidateTemplate",
    ]
    resources = [
      "arn:aws:cloudformation:*:${data.aws_caller_identity.current.account_id}:stack/baby-tracker/*",
      "arn:aws:cloudformation:*:${data.aws_caller_identity.current.account_id}:stack/aws-sam-cli-managed-default/*",
    ]
  }

  statement {
    sid    = "SamManagedBucket"
    effect = "Allow"
    actions = [
      "s3:CreateBucket",
      "s3:PutBucketPolicy",
      "s3:PutBucketVersioning",
      "s3:PutBucketPublicAccessBlock",
      "s3:PutEncryptionConfiguration",
      "s3:GetBucketLocation",
      "s3:GetObject",
      "s3:PutObject",
      "s3:ListBucket",
    ]
    resources = [
      "arn:aws:s3:::aws-sam-cli-managed-default-*",
      "arn:aws:s3:::aws-sam-cli-managed-default-*/*",
    ]
  }

  statement {
    sid    = "Lambda"
    effect = "Allow"
    actions = [
      "lambda:GetFunction",
      "lambda:CreateFunction",
      "lambda:DeleteFunction",
      "lambda:UpdateFunctionCode",
      "lambda:UpdateFunctionConfiguration",
      "lambda:GetFunctionConfiguration",
      "lambda:AddPermission",
      "lambda:RemovePermission",
      "lambda:GetPolicy",
      "lambda:ListVersionsByFunction",
      "lambda:PublishVersion",
      "lambda:TagResource",
      "lambda:UntagResource",
    ]
    resources = [
      "arn:aws:lambda:*:${data.aws_caller_identity.current.account_id}:function:dev-*",
    ]
  }

  statement {
    sid    = "HttpApi"
    effect = "Allow"
    actions = [
      "apigateway:GET",
      "apigateway:POST",
      "apigateway:PUT",
      "apigateway:PATCH",
      "apigateway:DELETE",
    ]
    resources = [
      "arn:aws:apigateway:${var.aws_region}::/apis",
      "arn:aws:apigateway:${var.aws_region}::/apis/*",
    ]
  }

  statement {
    sid    = "IamForFunctionExecutionRoles"
    effect = "Allow"
    actions = [
      "iam:CreateRole",
      "iam:DeleteRole",
      "iam:GetRole",
      "iam:AttachRolePolicy",
      "iam:DetachRolePolicy",
      "iam:PutRolePolicy",
      "iam:DeleteRolePolicy",
      "iam:GetRolePolicy",
      "iam:PassRole",
      "iam:TagRole",
    ]
    resources = [
      "arn:aws:iam::${data.aws_caller_identity.current.account_id}:role/baby-tracker-*",
    ]
  }

  statement {
    sid    = "Logs"
    effect = "Allow"
    actions = [
      "logs:CreateLogGroup",
      "logs:DeleteLogGroup",
      "logs:PutRetentionPolicy",
      "logs:DescribeLogGroups",
      "logs:TagResource",
    ]
    resources = [
      "arn:aws:logs:*:${data.aws_caller_identity.current.account_id}:log-group:/aws/lambda/dev-*",
    ]
  }
}

resource "aws_iam_role_policy" "sam_deploy" {
  name   = "SamDeploy"
  role   = aws_iam_role.github_actions_deploy.id
  policy = data.aws_iam_policy_document.sam_deploy.json
}
