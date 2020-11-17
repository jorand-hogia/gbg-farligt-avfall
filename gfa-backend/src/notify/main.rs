use std::{env, str::FromStr, collections::HashMap};
use lambda::{handler_fn, Context};
use serde_json::{json, Value};
use simple_logger::{SimpleLogger};
use log::{self, info, error, LevelFilter};
use rusoto_core::Region;
use rusoto_sns::{SnsClient, Sns, PublishInput, MessageAttributeValue};
use chrono::{Utc};
use common::events_repo::{get_by_date};

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
    let sns_client = SnsClient::new(region);
    for event in todays_events {
        let mut message_attributes: HashMap<String, MessageAttributeValue> = HashMap::new();
        message_attributes.insert("location_id".to_string(), MessageAttributeValue{
            data_type: "String".to_string(),
            string_value: Some(event.location_id),
            ..Default::default()
        });
        info!("Try publish event for: {}", event.street);
        let publish_input = PublishInput{
            message: format!("Farligt avfall-bilen arrives to {}, today at {}-{}", event.street, event.time_start, event.time_end),
            message_attributes: Some(message_attributes),
            topic_arn: Some(today_topic_arn.clone()),
            ..Default::default()
        };
        match sns_client.publish(publish_input).await {
            Ok(output) => {
                info!("Successfully published message");
                if output.message_id.is_some() {
                    info!("Message ID: {}", output.message_id.unwrap());
                }
            },
            Err(e) => {
                error!("Failed to publish message due to: {}", e);
            }
        };
    }

    Ok(json!({}))
}
