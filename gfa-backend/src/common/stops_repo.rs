use std::{collections::HashMap};
use rusoto_dynamodb::{DynamoDb, DynamoDbClient, ScanInput, AttributeValue};
use rusoto_core::{Region};
use log::{self, warn};
use crate::pickup_stop::PickUpStop;
use crate::dynamodb_util::MalformedDynamoDbResponse;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

pub async fn get_all_stops(table: &str, region: &Region, location_index: &str) -> Result<Vec<PickUpStop>, Error> {
    let client = DynamoDbClient::new(region.clone());
    let output = match client.scan(ScanInput{
        table_name: table.to_owned(),
        index_name: Some(location_index.to_owned()),
        ..Default::default()
    }).await {
        Ok(output) => output,
        Err(error) => return Err(Box::new(error)) 
    };
    let items = match output.items {
        Some(items) => items,
        None => return Err(Box::new(MalformedDynamoDbResponse))
    };
    let mut stops = items.iter()
        .map(|item| item_to_stop(item))
        .filter_map(|optional_stop| {
            if optional_stop.is_none() {
                warn!("Found malformed stop");
                return None;
            }
            Some(optional_stop.unwrap())
        })
        .collect::<Vec<PickUpStop>>();
    stops.sort();
    stops.dedup();
    Ok(stops)
}

fn item_to_stop(item: &HashMap<String, AttributeValue>) -> Option<PickUpStop> {
    let location_id = item.get("location_id")?.s.as_ref()?;
    let street = item.get("street")?.s.as_ref()?;
    let district = item.get("district")?.s.as_ref()?;
    let description = match item.get("description") {
        None => None,
        Some(description) => Some(description.s.as_ref()?.clone())
    };
    Some(PickUpStop{
        location_id: location_id.clone(),
        street: street.clone(),
        district: district.clone(),
        description,
    })
}
