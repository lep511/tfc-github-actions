#![allow(non_snake_case)]

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
use aws_sdk_dynamodb::{Client};
use serde::{Deserialize};
use serde_json;


#[derive(Deserialize, Debug)]
#[serde(rename_all="camelCase")]
pub struct Event {
    pub id: String,
    #[serde(rename = "detail-type")]
    pub detail_type: String,
    pub detail: Detail,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all="camelCase")]
pub struct Detail {
    pub data: Data,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all="camelCase")]
pub struct Data {
    pub orderData: OrderData,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all="camelCase")]
pub struct OrderData {
    pub sourceOrderId: String,
    pub items: Vec<Item>,
    pub shipments: Vec<Shipment>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all="camelCase")]
pub struct Component {
    pub code: String,
    pub fetch: bool,
    pub path: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all="camelCase")]
pub struct Item {
    pub sku: String,
    pub sourceItemId: String,
    pub components: Vec<Component>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all="camelCase")]
pub struct ShipTo {
    pub name: String,
    pub companyName: Option<String>,
    pub address1: String,
    pub town: String,
    pub postcode: String,
    pub isoCountry: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all="camelCase")]
pub struct Carrier {
    pub code: String,
    pub service: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all="camelCase")]
pub struct Shipment {
    pub shipTo: ShipTo,
    pub carrier: Carrier,
}

async fn process_record(message: &SqsMessage) -> Result<(), Error> {
    let mm = message.body.as_ref().unwrap();
    let event: Event = serde_json::from_str(&mm)?;
    let table_name =  env::var("DYNAMO_TABLE").expect("DYNAMO_TABLE must be set");
    let config = aws_config::load_defaults(BehaviorVersion::v2024_03_28()).await;
    let client = Client::new(&config);

    let event_id = event.id.clone();
    let detail_type = event.detail_type.clone();
    let source_order = event.detail.data.orderData.sourceOrderId.clone();
    let item_id = event.detail.data.orderData.items[0].sourceItemId.clone();
    let sku = event.detail.data.orderData.items[0].sku.clone();

    let s_event_id = AttributeValue::S(event_id);
    let s_detail_type = AttributeValue::S(detail_type);
    let s_source_order = AttributeValue::S(source_order);
    let s_item_id = AttributeValue::S(item_id);
    let s_sku = AttributeValue::S(sku);

    let request = client
        .put_item()
        .table_name(table_name)
        .item("SourceOrderID", s_source_order)
        .item("SourceItemID", s_item_id)
        .item("Sku", s_sku)
        .item("EventId", s_event_id)
        .item("DetailType", s_detail_type);

    let _resp = request.send().await?;

    Ok(())

}

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
/// - https://github.com/aws-samples/serverless-rust-demo/
async fn function_handler(event: LambdaEvent<SqsEvent>) -> Result<SqsBatchResponse, Error> {
    
    let mut batch_item_failures = Vec::new();

    //let _detail_type = event.payload.detail_type;
    //let _event_detail = event.payload.detail;
    // tracing::info!("Received event: {:?}", event_detail);

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
