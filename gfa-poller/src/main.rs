use lambda::{handler_fn, Context};
use simple_logger::{SimpleLogger};
use log::{self, error, debug, LevelFilter};
use std::env;
use futures::executor::block_on;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use core::str::FromStr;
use rusoto_core::{Region};

mod page_fetcher;
mod page_parser;
mod events_repo;
mod pickup_event;

#[derive(Deserialize)]
struct EmptyEvent {}

#[derive(Serialize)]
struct EmptyOutput {}

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

async fn handle_request(_event: Value, _c: Context) -> Result<String, Error> {
    let db_arn = env::var("DB_ARN").unwrap();
    let db_credentials = env::var("DB_CREDENTIALS").unwrap();
    let db_name = env::var("DB_NAME").unwrap();
    println!("{}\n{}\n{}", db_arn, db_credentials, db_name);
    let events_table = env::var("EVENTS_TABLE").unwrap();
    let region = env::var("AWS_REGION").unwrap();
    let region = Region::from_str(&region).unwrap(); 

    debug!("About to load pages");
    let pages_to_scrape = block_on(page_fetcher::obtain_pages());
    let pages_to_scrape = match pages_to_scrape {
        Ok(pages) => pages,
        Err(e) => {
            error!("{}", e);
            return Ok(e.to_string());
        }
    };
    debug!("Finished loading all pages");
    let mut all_events: Vec::<pickup_event::PickUpEvent> = Vec::new();
    for page in pages_to_scrape {
        let mut events = match page_parser::parse_page(page) {
            Ok(events) => events,
            Err(errors) => {
                for error in errors {
                    error!("{}", error);
                }
                return Ok("Failed while parsing pages".to_string());
            }
        };
        all_events.append(&mut events);
    }
    debug!("Finished parsing pages");

    let _result = match events_repo::store(events_table, region, all_events).await {
        Ok(res) => res,
        Err(e) => {
            error!("{}", e);
            return Ok(e.to_string());
        }
    };

    Ok("OK".to_string())
}
