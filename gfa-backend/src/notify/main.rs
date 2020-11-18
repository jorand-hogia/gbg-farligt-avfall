use std::{env, str::FromStr};
use lambda::{handler_fn, Context};
use serde_json::{json, Value};
use simple_logger::{SimpleLogger};
use log::{self, error, LevelFilter};
use rusoto_core::Region;
use chrono::{Utc};
use common::events_repo::{get_by_date};

mod publish_events;

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
    let today_topic_arn = env::var("TODAY_TOPIC").unwrap();
    let event_table = env::var("EVENTS_TABLE").unwrap();
    let region = env::var("AWS_REGION").unwrap();
    let region = Region::from_str(&region).unwrap(); 

    let todays_date = Utc::today().format("%Y-%m-%d").to_string();
    let todays_events = get_by_date(event_table, region.clone(), todays_date).await?;
    match publish_events::publish_events(todays_events, today_topic_arn, region).await {
        Err(e) => {
            error!("Errors occurred while publishing events, {}", e);
        },
        _ => {}
    };
    Ok(json!({}))
}
