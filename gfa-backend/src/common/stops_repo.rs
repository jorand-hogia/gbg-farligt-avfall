use std::{fmt, error, collections::HashMap};
use rusoto_dynamodb::{DynamoDb, DynamoDbClient, QueryInput, ScanInput, AttributeValue};
use rusoto_core::{Region};
use log::{self, warn};
use crate::pickup_stop::PickUpStop;
use crate::dynamodb_util::MalformedDynamoDbResponse;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug)]
struct MalformedStop {
    location_id: String,
}
impl fmt::Display for MalformedStop {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Malformed stop in database: {}", self.location_id)
  } 
}
impl error::Error for MalformedStop {}


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

pub async fn get_single_stop(table: &str, region: &Region, location_index: &str, location_id: &str) -> Result<Option<PickUpStop>, Error> {
    let client = DynamoDbClient::new(region.clone());
    let output = match client.query(QueryInput{
        table_name: table.to_owned(),
        index_name: Some(location_index.to_owned()),
        expression_attribute_values: Some([(":locationId".to_owned(), AttributeValue{s: Some(location_id.to_owned()), ..Default::default()})]
            .iter()
            .cloned()
            .collect()),
        key_condition_expression: Some("location_id = :locationId".to_owned()),
        limit: Some(1),
        ..Default::default()
    }).await {
        Ok(output) => output,
        Err(error) => return Err(Box::new(error))
    };
    let items = match output.items {
        Some(items) => items,
        None => return Err(Box::new(MalformedDynamoDbResponse))
    };
    if items.is_empty() {
        return Ok(None);
    }
    let item = items.first().unwrap();
    match item_to_stop(item) {
        Some(stop) => Ok(Some(stop)),
        None => Err(Box::new(MalformedStop{
            location_id: location_id.to_owned()
        }))
    }
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
