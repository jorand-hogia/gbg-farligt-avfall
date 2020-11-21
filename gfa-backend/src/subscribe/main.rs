use std::{error, env, str::FromStr, collections::HashMap};
use simple_logger::{SimpleLogger};
use log::{self, info, LevelFilter};
use serde_json::{Value};
use lambda::{handler_fn, Context};
use aws_lambda_events::event::apigw::{ApiGatewayProxyResponse, ApiGatewayProxyRequest};
use rusoto_core::{Region};

mod subscription;
mod subscribe;
mod validate_request;

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

async fn handle_request(event: ApiGatewayProxyRequest, _: Context) -> Result<ApiGatewayProxyResponse, Error> {
    info!("Subscribe start!");
    let today_topic_arn = env::var("TODAY_TOPIC").unwrap();
    let region = env::var("AWS_REGION").unwrap();
    let region = Region::from_str(&region).unwrap();
    let body = match event.body {
        Some(body) => body,
        None => return Ok(create_response("Missing request body".to_string(), 400)) 
    };
    let subscription: subscription::Subscription = match serde_json::from_str(&body) {
        Ok(subscription) => subscription,
        Err(e) => return Ok(create_response("Malformed request body".to_string(), 400))
    };
    match validate_request::validate(&subscription) {
        Err(e) => return Ok(create_response("Invalid request body".to_string(), 422)),
        _ => {}
    };
    match subscribe::subscribe(&subscription, today_topic_arn, region).await {
        Err(e) => return Ok(create_response("Failed to subscribe".to_string(), 500)),
        _ => {}
    };
    info!("{:?}", subscription);
    Ok(create_response("".to_string(), 200))
}

fn create_response(body: String, status_code: i64) -> ApiGatewayProxyResponse {
    let mut headers: HashMap<String, String> = HashMap::new();
    headers.insert("Access-Control-Allow-Headers".to_string(), "Content-Type,Accept".to_string());
    headers.insert("Access-Control-Allow-Methods".to_string(), "PUT".to_string());
    headers.insert("Access-Control-Allow-Origin".to_string(), "*".to_string());
    return ApiGatewayProxyResponse{
        status_code: status_code,
        headers: headers,
        multi_value_headers: HashMap::new(),
        body: Some(body),
        is_base64_encoded: None,
    }
}
