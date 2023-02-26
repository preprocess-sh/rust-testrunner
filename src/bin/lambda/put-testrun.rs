use lambda_http::{service_fn, Request};
use testrunner::{entrypoints::lambda::apigateway::{put_testrun}, utils::*};

type E = Box<dyn std::error::Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> Result<(), E> {
    // Initialize logger
    setup_tracing();

    // Initialize store
    let store = get_store().await;

    // Run the Lambda function
    lambda_http::run(service_fn(|event: Request| put_testrun(&store, event))).await?;
    Ok(())
}
