# DynamoDB Table
resource "random_pet" "table_name" {
  prefix    = "orders"
  separator = "_"
  length    = 4
}

resource "aws_dynamodb_table" "basic-dynamodb-table" {
  name           = random_pet.table_name.id
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

module "lambda_function" {
  source = "terraform-aws-modules/lambda/aws"

  function_name = "rust-aws-lambda"
  description   = "Create an AWS Lambda in Rust with Terraform"
  runtime       = "provided.al2"
  architectures = ["arm64"]
  handler       = "bootstrap"

  create_package         = false
  local_existing_package = "lambda-api/target/lambda/lambda-api/bootstrap.zip"
}