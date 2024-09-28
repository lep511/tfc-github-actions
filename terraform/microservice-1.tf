# Lambda - Rust runtime
module "lambda_function" {
  source = "terraform-aws-modules/lambda/aws"

  function_name = "${var.environment}-rust-aws-lambda"
  description   = "Create an AWS Lambda in Rust with Terraform"
  runtime       = "provided.al2023"
  architectures = ["x86_64"]
  handler       = "bootstrap"

  create_package         = false
  local_existing_package = "bootstrap.zip"
}


# DynamoDB Table
resource "aws_dynamodb_table" "basic-dynamodb-table" {
  name           = "${var.environment}-order-table"
  billing_mode   = "PROVISIONED"
  read_capacity  = 10
  write_capacity = 10
  hash_key       = "SourceOrderID"
  range_key      = "SourceItemID"

  attribute {
    name = "SourceOrderID"
    type = "S"
  }

  attribute {
    name = "SourceItemID"
    type = "S"
  }
}