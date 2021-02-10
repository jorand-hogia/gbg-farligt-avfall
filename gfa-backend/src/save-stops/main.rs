use std::env;
use std::str::FromStr;
use std::fmt;
use std::error;
use lambda::{handler_fn, Context};
use serde_json::{Value, to_vec};
use simple_logger::{SimpleLogger};
use log::{self, LevelFilter};
use rusoto_core::{Region, ByteStream};
use rusoto_s3::{S3, S3Client, PutObjectRequest};
use common::pickup_event::PickUpEvent;
use common::pickup_stop::PickUpStop;

mod stop_parser;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(fmt::Debug)]
pub struct SaveStopsError {
    pub message: String,
}
impl fmt::Display for SaveStopsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl error::Error for SaveStopsError {
    fn description(&self) -> &str {
        &self.message
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _log = SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init();
    let handler = handler_fn(handle_request);
    lambda::run(handler).await?;
    Ok(())
}

async fn handle_request(event: Value, _: Context) -> Result<(), Error> {
    let aws_region = env::var("AWS_REGION")?;
    let aws_region = Region::from_str(&aws_region)?;
    let stops_bucket = env::var("STOPS_BUCKET")?;
    let stops_path = env::var("STOPS_PATH")?;

    let pickup_events: Vec<PickUpEvent> = serde_json::from_value(event)?;
    let unique_stops: Vec<PickUpStop> = stop_parser::parse_unique_stops(pickup_events); 

    let mut request = PutObjectRequest::default();
    request.bucket = stops_bucket;
    request.key = stops_path;

    let body = to_vec(&unique_stops)?;
    request.body = Some(ByteStream::from(body));

    let s3_client = S3Client::new(aws_region);
    match s3_client.put_object(request).await {
        Ok(_res) => return Ok(()),
        Err(e) => return Err(Box::new(SaveStopsError{
            message: format!("Failed to write to S3: {}", e)
        }))
    };
}
