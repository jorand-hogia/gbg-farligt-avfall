use std::{env, fmt, error, str::FromStr, collections::HashMap};
use lambda::{handler_fn, Context};
use serde_json::{json, Value};
use simple_logger::{SimpleLogger};
use log::{self, debug, LevelFilter};
use rusoto_core::{Region};
use rusoto_s3::{S3, S3Client, GetObjectRequest};
use tokio::io::AsyncReadExt;
use aws_lambda_events::event::apigw::ApiGatewayProxyResponse;
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

async fn handle_request(_event: Value, _: Context) -> Result<ApiGatewayProxyResponse, Error> {
    debug!("Start get request");
    let aws_region = env::var("AWS_REGION")?;
    let aws_region = Region::from_str(&aws_region)?;
    let stops_bucket = env::var("STOPS_BUCKET")?;
    let stops_path = env::var("STOPS_PATH")?;

    let request = GetObjectRequest{
        bucket: stops_bucket,
        key: stops_path,
        ..Default::default()
    };

    debug!("About to make S3 request");
    let s3_client = S3Client::new(aws_region);
    let result = match s3_client.get_object(request).await {
        Ok(res) => res,
        Err(e) => return Err(Box::new(GetStopsError{
            message: format!("Failed to read from S3: {}", e),
        })),
    };
    debug!("Got response from S3");
    let body = match result.body {
        Some(body) => body,
        None => {
            let empty: Vec::<PickUpStop> = Vec::new();
            return Ok(create_response(json!(vec!(empty)).to_string()));
        }
    };
    debug!("Got body from S3");
    let mut response = String::new();
    body.into_async_read()
        .read_to_string(&mut response).await?;
    debug!("Got body: \n{}", response);
    return Ok(create_response(response));
}

fn create_response(body: String) -> ApiGatewayProxyResponse {
    let mut headers: HashMap<String, String> = HashMap::new();
    headers.insert("Access-Control-Allow-Headers".to_owned(), "Content-Type,Accept".to_owned());
    headers.insert("Access-Control-Allow-Methods".to_owned(), "GET".to_owned());
    headers.insert("Access-Control-Allow-Origin".to_owned(), "*".to_owned());
    ApiGatewayProxyResponse{
        status_code: 200,
        headers,
        multi_value_headers: HashMap::new(),
        body: Some(body),
        is_base64_encoded: None,
    }
}
