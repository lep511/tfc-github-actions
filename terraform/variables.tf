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

variable "version_app" {
  type        = string
  description = "The version of the application"
  default     = "0.1.0"
}

variable "aws_region" {
  type        = string
  description = "The AWS region to deploy the application"
  default     = "us-east-1"
}

variable "load_example_data" {
  description = "Flag: load example data into table items."
  type        = bool
  default     = true
}
