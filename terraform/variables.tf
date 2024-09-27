variable "region" {
  type        = string
  description = "The region to deploy resources to"
  default = "us-east-1"
}

# DynamoDB Table Name
variable "dynamo_table_name" {
  type        = string
  description = "DynamoDB Table Name"
  default = "GameScores"
}

variable "load_example_data" {
  description = "Flag: load example data into table items."
  type        = bool
  default     = true
}
