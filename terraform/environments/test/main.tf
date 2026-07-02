terraform {
  required_version = ">= 1.14.5"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "5.44.0"
    }
  }

  backend "s3" {
    encrypt = "true"
    bucket  = "terraform-state-prod-fancycoconut-backend"
    key     = "prod-fancycoconut-backend/terraform.tfstate"
    region  = var.region
  }
}

provider "aws" {
  region              = var.region
  allowed_account_ids = ["897729106526"]
}

module "babytrackerchip_github_repo_oidc" {
  source               = "../../modules/github-repo-oidc"
  aws_region           = var.region
  github_org           = "fancycoconut"
  github_repo          = "<repo-name-goes-here>"
  github_environment   = "<github-env-name-goes-here>"
  create_oidc_provider = true
}
