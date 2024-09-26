terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "4.52.0"
    }
    random = {
      source  = "hashicorp/random"
      version = "3.4.3"
    }
  }
  required_version = ">= 1.1.0"
}

provider "aws" {
  region = "us-east-2"
}

resource aws_iam_policy this {
  name        = format("%s-trigger-transcoder", "test")
  description = "Allow to access base resources and trigger transcoder"
  policy      = jsonencode(
    {
        "Version": "2012-10-17",
        "Statement": [
            {
                "Sid": "SomeVeryDefaultAndOpenActions",
                "Effect": "Allow",
                "Action": [
                    "logs:CreateLogGroup",
                    "ec2:DescribeNetworkInterfaces",
                    "ec2:CreateNetworkInterface",
                    "ec2:DeleteNetworkInterface",
                    "logs:CreateLogStream",
                    "logs:PutLogEvents"
                ],
                "Resource": [
                    "*"
                ]
            }
        ]
    })
}

resource "aws_iam_user_policy_attachment" "lambda-service-user-policy-attachment" {
  user       = aws_iam_user.lambda-service-user.name
  policy_arn = aws_iam_policy.this.arn
}
