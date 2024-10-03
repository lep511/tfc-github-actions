# Configure the AWS Provider
provider "aws" {
  region = var.aws_region
  default_tags {
    tags = {
      environment     = var.environment
      owner           = "Ops"
      applicationName = var.application_name
      awsApplication  = aws_servicecatalogappregistry_application.terraform_app.application_tag.awsApplication
    }    
  }
  
  # Make it faster by skipping something
  # skip_metadata_api_check     = true
  # skip_region_validation      = true
  # skip_credentials_validation = true
}

# Create application using aliased 'application' provider
provider "aws" {
  alias = "application"
  region = var.aws_region
}

# Register new application
# An AWS Service Catalog AppRegistry Application is displayed in the AWS Console under "MyApplications".
resource "aws_servicecatalogappregistry_application" "terraform_app" {
  provider    = aws.application
  name        = var.application_name
  description = "New sample terraform application"
}


module "eventbridge" {
  source = "terraform-aws-modules/eventbridge/aws"

  bus_name = "${var.environment}-order-bus"

  attach_sqs_policy = true
  attach_lambda_policy = true

  lambda_target_arns   = [
    module.lambda.lambda_function_arn
  ]

  sqs_target_arns = [
    aws_sqs_queue.queue.arn,
    aws_sqs_queue.dlq.arn
  ]

  rules = {
    orders_create = {
      description = "Capture all created orders",
      event_pattern = jsonencode({
        "detail-type" : ["orderCreate"],
        "source" : ["api.gateway.orders.create"]
      })
    }
  }

  targets = {
    orders_create = [
      {
        name            = "send-orders-to-sqs"
        arn             = aws_sqs_queue.queue.arn
        dead_letter_arn = aws_sqs_queue.dlq.arn
        target_id       = "send-orders-to-sqs"
      },
      {
        name            = "send-orders-to-lambda"
        arn             = module.lambda.lambda_function_arn
        target_id       = "send-orders-to-lambda"
      }
    ]
  }
}
##################
# Lambda [Rust]
##################
module "lambda" {
  source = "terraform-aws-modules/lambda/aws"

  function_name = "${var.environment}-rust-aws-lambda"
  description   = "Create an AWS Lambda in Rust with Terraform"
  runtime       = "provided.al2023"
  architectures = ["x86_64"]
  handler       = "bootstrap"

  create_package         = false
  local_existing_package = "bootstrap.zip"

  environment_variables = {
    DYNAMO_TABLE = aws_dynamodb_table.basic-dynamodb-table.name
  }

  attach_policy_json = true
  policy_json        = <<-EOT
    {
      "Version": "2012-10-17",
      "Statement": [
          {
            "Sid": "DynamoDBPutItem",
            "Effect": "Allow",
            "Action": [
                "dynamodb:PutItem"
            ],
            "Resource": "${aws_dynamodb_table.basic-dynamodb-table.arn}"
          },
          {
            "Sid": SQSManage",
            "Effect": "Allow",
            "Action": [
                "sqs:*"
            ],
            "Resource": "${aws_sqs_queue.queue.arn}"
          }
      ]
    }
  EOT

  create_current_version_allowed_triggers = false
  allowed_triggers = {
    ScanAmiRule = {
      principal  = "sqs.amazonaws.com"
      source_arn = aws_sqs_queue.queue.arn
    }
  }
}

##################
# DynamoDB Table
##################
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

##################
# Extra resources
##################

module "api_gateway" {
  source  = "terraform-aws-modules/apigateway-v2/aws"
  version = "~> 4.0"

  name          = "${var.environment}-http-api"
  description   = "My HTTP API Gateway"
  protocol_type = "HTTP"

  create_api_domain_name = false

  integrations = {
    "POST /orders/create" = {
      integration_type    = "AWS_PROXY"
      integration_subtype = "EventBridge-PutEvents"
      credentials_arn     = module.apigateway_put_events_to_eventbridge_role.iam_role_arn

      request_parameters = jsonencode({
        EventBusName = module.eventbridge.eventbridge_bus_name,
        Source       = "api.gateway.orders.create",
        DetailType   = "orderCreate",
        Detail       = "$request.body",
        Time         = "$context.requestTimeEpoch"
      })

      payload_format_version = "1.0"
    }
  }
}

module "apigateway_put_events_to_eventbridge_role" {
  source  = "terraform-aws-modules/iam/aws//modules/iam-assumable-role"
  version = "~> 4.0"

  create_role = true

  role_name         = "${var.environment}-apigateway-put-events-to-eventbridge"
  role_requires_mfa = false

  trusted_role_services = ["apigateway.amazonaws.com"]

  custom_role_policy_arns = [
    module.apigateway_put_events_to_eventbridge_policy.arn
  ]
}

module "apigateway_put_events_to_eventbridge_policy" {
  source  = "terraform-aws-modules/iam/aws//modules/iam-policy"
  version = "~> 4.0"

  name        = "${var.environment}-apigateway-put-events-to-eventbridge"
  description = "Allow PutEvents to EventBridge"

  policy = data.aws_iam_policy_document.apigateway_put_events_to_eventbridge_policy.json
}

data "aws_iam_policy_document" "apigateway_put_events_to_eventbridge_policy" {
  statement {
    sid       = "AllowPutEvents"
    actions   = ["events:PutEvents"]
    resources = [module.eventbridge.eventbridge_bus_arn]
  }

  depends_on = [module.eventbridge]
}

resource "aws_sqs_queue" "dlq" {
  name = "${var.environment}-orderqueue-dlq"
}

resource "aws_sqs_queue" "queue" {
  name = "${var.environment}-orderqueue"
}

resource "aws_lambda_event_source_mapping" "event_source_mapping" {
  batch_size        = 10
  event_source_arn  = "${aws_sqs_queue.queue.arn}"
  enabled           = true
  function_name     = module.lambda.lambda_function_arn
}

resource "aws_sqs_queue_policy" "queue" {
  queue_url = aws_sqs_queue.queue.id
  policy    = data.aws_iam_policy_document.queue.json
}

data "aws_iam_policy_document" "queue" {
  statement {
    sid     = "AllowSendMessage"
    actions = ["sqs:SendMessage"]

    principals {
      type        = "Service"
      identifiers = ["events.amazonaws.com"]
    }

    resources = [aws_sqs_queue.queue.arn]
  }
}