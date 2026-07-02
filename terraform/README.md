# Infrastructure as Code

## Getting Started

Follow the installation guide [here](https://developer.hashicorp.com/terraform/tutorials/aws-get-started/install-cli).

Set up your AWS CLI.

Then we will add / store our AWS credentials.

```bash
# We will configure a profile for our credentials
$ aws configure --profile fancycoconut
Enter Access Key Id: ABDCDEFDASDASF
Enter Secret Key: %%%
```

After this is done, we can run the following command to list all our S3 buckets to show that it works.

```bash
$ aws s3 ls --profile fancycoconut

# To see which AWS Identity is active
$ aws sts get-caller-identity
```

### Initializing Terraform

Goto the `prod` folder under `terraform` > `environments`. Ensure there is a corresponding S3 bucket for the environment state e.g. `terraform-state-fancycoconut-prod` that was created beforehand. Then run

```bash
# We set the AWS_PROFILE environment variable to ensure Terraform uses the correct credentials
export AWS_PROFILE=fancycoconut
$ terraform init
```

To generate a plan of your changes, run

```bash
# We set the AWS_PROFILE environment variable to ensure Terraform uses the correct credentials
export AWS_PROFILE=fancycoconut
terraform plan -out temp.plan -var 'environment=prod'
```

Once you have reviewed the plan and everything looks good, you can apply the plan with:

```bash
# We set the AWS_PROFILE environment variable to ensure Terraform uses the correct credentials
export AWS_PROFILE=fancycoconut
terraform apply "temp.plan"
```
