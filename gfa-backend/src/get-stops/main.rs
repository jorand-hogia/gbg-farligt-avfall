use std::env;
use std::str::FromStr;
use lambda::{handler_fn, Context};
use serde_json::{json, Value, Deserializer};
use simple_logger::{SimpleLogger};
use log::{self, LevelFilter};
use rusoto_core::{Region};
use rusoto_s3::{S3, S3Client, GetObjectRequest};
use serde::Deserialize;
use common::pickup_stop::PickUpStop;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _log = SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init();
    let handler = handler_fn(handle_request);
    lambda::run(handler).await?;
    Ok(())
}

async fn handle_request(_event: Value, _: Context) -> Result<Value, Error> {
    let aws_region = env::var("AWS_REGION")?;
    let aws_region = Region::from_str(&aws_region)?;
    let stops_bucket = env::var("STOPS_BUCKET")?;
    let stops_path = env::var("STOPS_PATH")?;

    let mut request = GetObjectRequest::default();
    request.bucket = stops_bucket;
    request.key = stops_path;

    let s3_client = S3Client::new(aws_region);
    let result = s3_client.get_object(request).await?;
    let body = match result.body {
        Some(body) => body.into_blocking_read(),
        None => {
            let empty: Vec::<PickUpStop> = Vec::new();
            return Ok(json!(vec!(empty)));
        }
    };
    let mut a = Deserializer::from_reader(body);
    let b = Vec::<PickUpStop>::deserialize(&mut a)?;
    return Ok(json!(b));
}
