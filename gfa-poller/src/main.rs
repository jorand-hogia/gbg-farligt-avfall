use std::error::Error;

use lambda_runtime::{error::HandlerError, lambda, Context};
use simple_logger::{SimpleLogger};
use log::{self, info, LevelFilter};
use serde::{Deserialize, Serialize};

mod url_parser;

#[derive(Deserialize)]
struct EmptyEvent {}

#[derive(Serialize)]
struct EmptyOutput {}

fn main() -> Result<(), Box<dyn Error>> {
    let _log = SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init();
    lambda!(handle_request);
    Ok(())
}

fn handle_request(_e: EmptyEvent, _c: Context) -> Result<EmptyOutput, HandlerError> {
    info!("Hello farligt avfall!");
    let urls_to_scrape = url_parser::obtain_urls();
    Ok(EmptyOutput{})
}
