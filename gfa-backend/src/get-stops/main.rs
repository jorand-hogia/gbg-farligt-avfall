use std::{env, fmt, error, str::FromStr, collections::HashMap};
use lambda::{handler_fn, Context};
use serde_json::{Value};
use simple_logger::{SimpleLogger};
use log::{self, LevelFilter};
use rusoto_core::{Region};
use aws_lambda_events::event::apigw::ApiGatewayV2httpResponse;
use common::stops_repo::get_all_stops;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(fmt::Debug)]
pub struct GetStopsError {
    pub message: String,
}
impl fmt::Display for GetStopsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl error::Error for GetStopsError {
    fn description(&self) -> &str {
        &self.message
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _log = SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        .init();
    let handler = handler_fn(handle_request);
    lambda::run(handler).await?;
    Ok(())
}

async fn handle_request(_event: Value, _: Context) -> Result<ApiGatewayV2httpResponse, Error> {
    let aws_region = env::var("AWS_REGION")?;
    let aws_region = Region::from_str(&aws_region)?;
    let events_table = env::var("EVENTS_TABLE")?;
    let location_index = env::var("LOCATION_INDEX")?;

    let stops = match get_all_stops(&events_table, &aws_region, &location_index).await {
        Ok(stops) => stops, 
        Err(e) => {
            return Ok(create_response(format!("Failed to read stops: {}", e), 500))
        }
    };
    let stops_json = match serde_json::to_string(&stops) {
        Ok(json) => json,
        Err(e) => {
            return Ok(create_response(format!("Failed to serialize stops into json: {}", e), 500))
        }
    };
    Ok(create_response(stops_json, 200))
}

fn create_response(body: String, status_code: i64) -> ApiGatewayV2httpResponse {
    ApiGatewayV2httpResponse {
        status_code,
        headers: HashMap::new(),
        multi_value_headers: HashMap::new(),
        body: Some(body),
        is_base64_encoded: Some(false),
        cookies: Vec::new()
    }
}
