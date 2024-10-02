# The region is specified in  GitHub - settings/variables/actions
variable "application_name" {
  type        = string
  description = "The name of the application"
  default     = "TerraformApp-stage"
}

variable "environment" {
  type        = string
  description = "The name of the environment"
  default     = "stage"
}

variable "region" {
  type        = string
  description = "The AWS region to deploy the application"
  default     = "us-east-1"
}
