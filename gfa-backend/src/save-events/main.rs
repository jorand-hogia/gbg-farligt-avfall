use std::{env, str::FromStr};
use lambda::{handler_fn, Context};
use serde_json::{json, Value};
use simple_logger::{SimpleLogger};
use log::{self, info, error, LevelFilter};
use rusoto_core::Region;
use chrono::{Utc};
use common::{pickup_event::PickUpEvent, events_repo};

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

mod events_filter;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _log = SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init();
    let handler = handler_fn(handle_request);
    lambda::run(handler).await?;
    Ok(())
}

async fn handle_request(event: Value, _: Context) -> Result<Value, Error> {
    let events_table = env::var("EVENTS_TABLE").unwrap();
    let region = env::var("AWS_REGION").unwrap();
    let region = Region::from_str(&region).unwrap(); 
    let pickup_events: Vec<PickUpEvent> = serde_json::from_value(event)?;
    let pickup_events = events_filter::filter(pickup_events, Utc::today());
    let total_events = pickup_events.len();
    // Filter events
    // Skip in the past, long time in the future, etc

    let _result = match events_repo::store(events_table, region, pickup_events).await {
        Ok(unprocessed_count) => {
            let processed_items = total_events - unprocessed_count;
            info!("Wrote {} / {} items", processed_items, total_events);
        },
        Err(e) => {
            error!("Error when writing events:\n {}", e);
            return Err(e);
        }
    };

    Ok(json!({}))
}
