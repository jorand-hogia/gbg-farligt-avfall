use std::{env, fmt, error, str::FromStr, collections::HashMap, io::Read};
use lambda::{handler_fn, Context};
use serde_json::{json, Value};
use simple_logger::{SimpleLogger};
use log::{self, debug, LevelFilter};
use rusoto_core::{Region};
use rusoto_s3::{S3, S3Client, GetObjectRequest};
use aws_lambda_events::event::apigw::ApiGatewayV2httpResponse;
use common::pickup_stop::PickUpStop;

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
    let stops_bucket = env::var("STOPS_BUCKET")?;
    let stops_path = env::var("STOPS_PATH")?;

    let request = GetObjectRequest{
        bucket: stops_bucket,
        key: stops_path,
        ..Default::default()
    };

    let s3_client = S3Client::new(aws_region);
    let result = match s3_client.get_object(request).await {
        Ok(res) => res,
        Err(e) => return Err(Box::new(GetStopsError{
            message: format!("Failed to read from S3: {}", e),
        })),
    };
    let body = match result.body {
        Some(body) => body,
        None => {
            let empty: Vec::<PickUpStop> = Vec::new();
            return Ok(create_response(json!(vec!(empty)).to_string()));
        }
    };
    let mut response = String::new();
    match body.into_blocking_read()
        .read_to_string(&mut response) {
            Ok(_size) => {},
            Err(e) => return Err(Box::new(GetStopsError{
                message: format!("Could not transform S3 response into string: {}", e),
            })),
        }
    debug!("Got body: \n{}", response);
    return Ok(create_response(response));
}

fn create_response(body: String) -> ApiGatewayV2httpResponse {
    ApiGatewayV2httpResponse {
        status_code: 200,
        headers: HashMap::new(),
        multi_value_headers: HashMap::new(),
        body: Some(body),
        is_base64_encoded: Some(false),
        cookies: Vec::new()
    }
}
