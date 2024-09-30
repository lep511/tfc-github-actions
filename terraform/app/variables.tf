# The region is specified in  GitHub - settings/variables/actions
variable "application_name" {
  type        = string
  description = "The name of the application"
  default     = "TerraformApp-dev"
}

variable "environment" {
  type        = string
  description = "The name of the environment"
  default     = "dev"
}

variable "region" {
  type        = string
  description = "The AWS region to deploy the application"
  default     = "us-east-1"
}
