variable "environment" {
  description = "the name of your environment, e.g. \"test\""
  type        = string
  default     = "prod"
}

variable "region" {
  description = "The region to use"
  type        = string
  default     = "ap-southeast-2"
}
