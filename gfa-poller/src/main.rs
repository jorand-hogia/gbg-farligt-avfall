use std::error::Error;

use lambda_runtime::{error::HandlerError, lambda, Context};
use simple_logger::{SimpleLogger};
use log::{self, info, LevelFilter};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct EmptyEvent {}

#[derive(Serialize)]
struct EmptyOutput {}

fn main() -> Result<(), Box<dyn Error>> {
    SimpleLogger::new()
        .with_level(LevelFilter::Info);
    lambda!(handle_request);
    Ok(())
}

fn handle_request(_e: EmptyEvent, _c: Context) -> Result<EmptyOutput, HandlerError> {
    info!("Hello farligt avfall!");
    Ok(EmptyOutput{})
}
