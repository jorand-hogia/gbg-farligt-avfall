use std::error::Error;
use lambda_runtime::{error::HandlerError, lambda, Context};
use simple_logger::{SimpleLogger};
use futures::executor::block_on;
use log::{self, error, LevelFilter};
use serde::{Deserialize, Serialize};

mod page_fetcher;
mod page_parser;

#[derive(Deserialize)]
struct EmptyEvent {}

#[derive(Serialize)]
struct EmptyOutput {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _log = SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init();
    lambda!(handle_request);
    Ok(())
}

fn handle_request(_e: EmptyEvent, _c: Context) -> Result<EmptyOutput, HandlerError> {
    let pages_to_scrape = block_on(page_fetcher::obtain_pages());
    let pages_to_scrape = match pages_to_scrape {
        Ok(pages) => pages,
        Err(e) => {
            error!("{}", e);
            return Ok(EmptyOutput{});
        }
    };
    let mut all_events: Vec::<page_parser::PickUpEvent> = Vec::new();
    for page in pages_to_scrape {
        let mut events = match page_parser::parse_page(page) {
            Ok(events) => events,
            Err(errors) => {
                for error in errors {
                    error!("{}", error);
                }
                return Ok(EmptyOutput{});
            }
        };
        all_events.append(&mut events);
    }
    for event in all_events {
        println!("{}", event);
    }
    Ok(EmptyOutput{})
}
