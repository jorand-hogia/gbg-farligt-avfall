use lambda::{handler_fn, Context};
use simple_logger::{SimpleLogger};
use log::{self, error, debug, LevelFilter};
use futures::executor::block_on;
use serde_json::{json, Value};
use std::fmt;
use std::error;
use common::pickup_event::PickUpEvent;

mod page_fetcher;
mod page_parser;

#[derive(fmt::Debug)]
pub struct GfaScraperError {
    pub message: String,
}
impl fmt::Display for GfaScraperError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl error::Error for GfaScraperError {
    fn description(&self) -> &str {
        &self.message
    }
}

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

async fn handle_request(_event: Value, _c: Context) -> Result<Value, Error> {
    debug!("About to load pages");
    let pages_to_scrape = block_on(page_fetcher::obtain_pages());
    let pages_to_scrape = match pages_to_scrape {
        Ok(pages) => pages,
        Err(e) => {
            error!("{}", e);
            return Err(Box::new(GfaScraperError{
                message: format!("Failed while fetching pages: {}", e)
            })) 
        }
    };
    debug!("Finished loading all pages");
    let mut all_events: Vec::<PickUpEvent> = Vec::new();
    for page in pages_to_scrape {
        let mut events = match page_parser::parse_page(page) {
            Ok(events) => events,
            Err(error) => {
                error!("{}", error);
                return Err(Box::new(GfaScraperError{
                    message: format!("Failed while parsing pages")
                })) 
            }
        };
        all_events.append(&mut events);
    }
    debug!("Finished parsing pages");
    Ok(json!(all_events))
}
