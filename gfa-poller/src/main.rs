use std::error::Error;

use lambda_runtime::{error::HandlerError, lambda, Context};
use simple_logger::{SimpleLogger};
use log::{self, info, LevelFilter};
use serde::{Deserialize, Serialize};

mod page_fetcher;

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
    let pages_to_scrape = page_fetcher::obtain_pages().unwrap();
    for p in pages_to_scrape {
        println!("{}", p.len());
    }
    Ok(EmptyOutput{})
}
