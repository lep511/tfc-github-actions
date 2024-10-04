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
