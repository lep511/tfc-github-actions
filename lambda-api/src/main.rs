use std::env;
use aws_lambda_events::{
    event::dynamodb::{StreamRecord},
    event::sqs::{SqsBatchResponse},
    sqs::{BatchItemFailure, SqsEventObj},
    dynamodb::{EventRecord}
};
use aws_sdk_dynamodb::types::AttributeValue;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use aws_config::BehaviorVersion;
use serde::Deserialize;

use aws_sdk_dynamodb::{
    Client, 
    Error as DynamoError
};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    id: String,
    name: String,
    description: String,
    custom_note: String
}

pub async fn add_item(client: &Client, item: Item, table: &String) -> Result<(), DynamoError> {
    let s_id = AttributeValue::S(item.id);
    let s_name = AttributeValue::S(item.name);
    let s_description = AttributeValue::S(item.description);
    let s_custom_note = AttributeValue::S(item.custom_note);

    let request = client
        .put_item()
        .table_name(table)
        .item("id", s_id)
        .item("name", s_name)
        .item("description", s_description)
        .item("note", s_custom_note);

    let _resp = request.send().await?;

    Ok(())
}


async fn process_record(stream: StreamRecord) -> Result<(), Error> {
    let table  =  env::var("DYNAMO_TABLE").expect("DYNAMO_TABLE must be set");   
    let config = aws_config::load_defaults(BehaviorVersion::v2024_03_28()).await;
    let client = aws_sdk_dynamodb::Client::new(&config);
    
    let item: Item = serde_dynamo::from_item(stream.new_image.into_inner()).expect("(Error) Unwrapping Item");
    tracing::info!("{:?}", item);
    println!("Id = {}", item.id);

    let _ = add_item(&client, item, &table.to_string()).await;

    Ok(())

}

// function_handler
// Lambda handler code for responding to events read from SQS
async fn function_handler(event: LambdaEvent<SqsEventObj<EventRecord>>) -> Result<SqsBatchResponse, Error> {
    let mut batch_item_failures = Vec::new();

    for r in event.payload.records {
        match process_record(r.body.change).await {
            Ok(_) => (),
            Err(_) => batch_item_failures.push(BatchItemFailure {
                item_identifier: r.message_id.unwrap(),
            }),
        }
    }

    Ok(SqsBatchResponse {
        batch_item_failures,
    })
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .json()
        //.pretty()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()

        .init();

    run(service_fn(function_handler)).await
}