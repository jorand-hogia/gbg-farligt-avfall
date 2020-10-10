use lambda::handler_fn;
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

async fn handle_request(_event: String, _c: lambda::Context) -> Result<String, Error> {
    let pages_to_scrape = block_on(page_fetcher::obtain_pages());
    let pages_to_scrape = match pages_to_scrape {
        Ok(pages) => pages,
        Err(e) => {
            error!("{}", e);
            return Ok(e.to_string());
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
                return Ok("Failed while parsing pages".to_string());
            }
        };
        all_events.append(&mut events);
    }
    for event in all_events {
        println!("{}", event);
    }
    Ok("OK".to_string())
}
