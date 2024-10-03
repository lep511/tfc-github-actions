use std::env;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};
// use serde_json::{json, Value};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use aws_sdk_dynamodb::types::AttributeValue;
use aws_config::BehaviorVersion;
use aws_lambda_events::{
    event::sqs::{SqsBatchResponse, SqsEvent},
    sqs::{BatchItemFailure, SqsMessage},
};
use aws_sdk_dynamodb::{
    Client, 
    Error as DynamoError
};

pub struct Item {
    pub source_order: String,
    pub source_item: String,
    pub sku: String,
}

pub async fn add_item(client: &Client, item: Item, table: &String) -> Result<(), DynamoError> {
    let s_order = AttributeValue::S(item.source_order);
    let s_item = AttributeValue::S(item.source_item);
    let s_sku = AttributeValue::S(item.sku);

    let request = client
        .put_item()
        .table_name(table)
        .item("SourceOrderID", s_order)
        .item("SourceItemID", s_item)
        .item("Sku", s_sku);

    // println!("Executing request [{request:?}] to add item...");

    let _resp = request.send().await?;

    Ok(())
}

async fn process_record(message: &SqsMessage) -> Result<(), Error> {
    println!("Process record: {:?}", message);
    Err(Error::from("Error processing message"))
}

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
/// - https://github.com/aws-samples/serverless-rust-demo/
async fn function_handler(event: LambdaEvent<SqsEvent>) -> Result<SqsBatchResponse, Error> {
    let table_name =  env::var("DYNAMO_TABLE").expect("DYNAMO_TABLE must be set");
    let mut batch_item_failures = Vec::new();

    //let _detail_type = event.payload.detail_type;
    //let _event_detail = event.payload.detail;
    
    let config = aws_config::load_defaults(BehaviorVersion::v2024_03_28()).await;
    let client = aws_sdk_dynamodb::Client::new(&config);

    // tracing::info!("Received event: {:?}", event_detail);

    let item = Item {
        source_order: "testuser".into(),
        source_item: "Brown".into(),
        sku: "odata-27".into(),
    };

    let resp = add_item(&client, item, &table_name.to_string()).await;

    println!("Response: {:?}", resp);

    for record in event.payload.records {
        match process_record(&record).await {
            Ok(_) => (),
            Err(_) => batch_item_failures.push(BatchItemFailure {
                item_identifier: record.message_id.unwrap(),
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
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
