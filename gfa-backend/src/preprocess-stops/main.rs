use lambda::{handler_fn, Context};
use serde_json::{Value};
use simple_logger::{SimpleLogger};
use log::{self, info, LevelFilter};
use common::pickup_event::PickUpEvent;
use common::pickup_stop::PickUpStop;

mod stop_parser;

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

async fn handle_request(event: Value, _: Context) -> Result<Value, Error> {
    let pickup_events: Vec<PickUpEvent> = serde_json::from_value(event).unwrap();
    let unique_stops: Vec<PickUpStop> = stop_parser::parse_unique_stops(pickup_events); 
    for stop in unique_stops {
        println!("{}", stop);
    }
    Ok(Value::String("OK".to_string()))
}
