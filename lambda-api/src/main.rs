use lambda_runtime::{service_fn, LambdaEvent, Error};
use tracing_subscriber::filter::{EnvFilter, LevelFilter};
use serde_json::{json, Value};
// use aws_sdk_eventbridge as eventbridge;


async fn handler(event: LambdaEvent<Value>) -> Result<Value, Error> {
    // Log the entire event
    println!("Event: {:?}", event.payload);
    
    let payload = event.payload;
    let first_name = payload["firstName"].as_str().unwrap_or("world");
    
    Ok(json!({ "message": format!("Hello, {first_name}!") }))
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
    
    //let config = aws_config::load_from_env().await;
    //let client = aws_sdk_eventbridge::Client::new(&config);

    lambda_runtime::run(service_fn(handler)).await
}