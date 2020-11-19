use std::{error, collections::HashMap};
use simple_logger::{SimpleLogger};
use log::{self, info, LevelFilter};
use serde_json::{Value};
use lambda::{handler_fn, Context};
use aws_lambda_events::event::apigw::ApiGatewayProxyResponse;

type Error = Box<dyn error::Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _log = SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init();
    let handler = handler_fn(handle_request);
    lambda::run(handler).await?;
    Ok(())
}

async fn handle_request(_event: Value, _: Context) -> Result<ApiGatewayProxyResponse, Error> {
    info!("Subscribe start!");
    Ok(create_response("".to_string()))
}

fn create_response(body: String) -> ApiGatewayProxyResponse {
    let mut headers: HashMap<String, String> = HashMap::new();
    headers.insert("Access-Control-Allow-Headers".to_string(), "Content-Type,Accept".to_string());
    headers.insert("Access-Control-Allow-Methods".to_string(), "PUT".to_string());
    headers.insert("Access-Control-Allow-Origin".to_string(), "*".to_string());
    return ApiGatewayProxyResponse{
        status_code: 200,
        headers: headers,
        multi_value_headers: HashMap::new(),
        body: Some(body),
        is_base64_encoded: None,
    }
}
