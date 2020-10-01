use std::error::Error;

use lambda_runtime::{error::HandlerError, lambda, Context};
use simple_logger::{SimpleLogger};
use log::{self, info, warn, LevelFilter};
use serde::{Deserialize, Serialize};

mod page_fetcher;
mod page_parser;

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
    let mut all_events: Vec::<page_parser::PickUpEvent> = Vec::new();
    for page in pages_to_scrape {
        let mut events = match page_parser::parse_page(page) {
            Ok(events) => events,
            Err(e) => {
                warn!("{}", e.message);
                Vec::new()
            }
        };
        all_events.append(&mut events);
    }
    for event in all_events {
        println!("{}", event);
    }
    Ok(EmptyOutput{})
}
