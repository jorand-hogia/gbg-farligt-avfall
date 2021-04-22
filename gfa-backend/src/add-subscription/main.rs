use std::{collections::HashMap};
use lambda::{handler_fn, Context};
use log::{self, LevelFilter};
use simple_logger::{SimpleLogger};
use serde_json::{Value};
use aws_lambda_events::event::apigw::ApiGatewayProxyResponse;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _log = SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        .init();
    let handler = handler_fn(handle_request);
    lambda::run(handler).await?;
    Ok(())
}

async fn handle_request(_event: Value, _: Context) -> Result<ApiGatewayProxyResponse, Error> {
    return Ok(create_response(String::new()));
}

fn create_response(body: String) -> ApiGatewayProxyResponse {
    let mut headers: HashMap<String, String> = HashMap::new();
    headers.insert("Access-Control-Allow-Headers".to_string(), "Content-Type,Accept".to_string());
    headers.insert("Access-Control-Allow-Methods".to_string(), "GET".to_string());
    headers.insert("Access-Control-Allow-Origin".to_string(), "*".to_string());
    return ApiGatewayProxyResponse{
        status_code: 200,
        headers: headers,
        multi_value_headers: HashMap::new(),
        body: Some(body),
        is_base64_encoded: None,
    }
}
