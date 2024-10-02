use aws_lambda_events::event::eventbridge::EventBridgeEvent;
use lambda_runtime::{run, service_fn, tracing, Error, LambdaEvent};
use tracing_subscriber::filter::{EnvFilter, LevelFilter};
use serde_json::{json, Value};

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
/// - https://github.com/aws-samples/serverless-rust-demo/
async fn function_handler(event: LambdaEvent<EventBridgeEvent>) -> Result<Value, Error> {
    // Extract some useful information from the request
    let _detail_type = event.payload.detail_type;
    let event_detail = event.payload.detail;
    tracing::info!("Received event: {:?}", event_detail);

    Ok(json!({ "message": format!("Hello world!") }))
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
