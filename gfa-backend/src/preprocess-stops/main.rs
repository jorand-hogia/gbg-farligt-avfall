use lambda::{handler_fn, Context};
use std::collections::HashMap;
use serde_json::{json, Value};
use simple_logger::{SimpleLogger};
use log::{self, info, LevelFilter};
use common::pickup_event::PickUpEvent;
use common::pickup_stop::PickUpStop;
use geocoder::GeoCoder;
use mapquest_geocoder::MapQuestGeoCoder;

mod stop_parser;
mod geocoder;
mod mapquest_geocoder;

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

// TODO: Handle this way more cleanly
async fn handle_request(event: Value, _: Context) -> Result<Value, Error> {
    let pickup_events: Vec<PickUpEvent> = serde_json::from_value(event)?;
    let unique_stops: Vec<PickUpStop> = stop_parser::parse_unique_stops(pickup_events); 
    let mut id_by_address: HashMap<String, String> = HashMap::new();
    for stop in unique_stops.iter() {
        id_by_address.insert(format!("{},{}", stop.street.clone(), stop.district.clone()), stop.location_id.clone());
    }
    let coordinatesMap = MapQuestGeoCoder::forward_geocode("2FSWyWz0ouHBnucVBWA80zsPb6K5wfwc".to_string(), id_by_address)?;
    let mut stops_with_coordinates: Vec<PickUpStop> = Vec::new();
    for stop in unique_stops.iter() {
        let coordinate = match coordinatesMap.get(&stop.location_id) {
            Some(coordinate) => {
                stops_with_coordinates.push(PickUpStop::from(stop, coordinate.clone()));
            },
            None => {
                stops_with_coordinates.push(PickUpStop::from(stop, None));
            } 
        };
    }
    Ok(json!(stops_with_coordinates))
}
