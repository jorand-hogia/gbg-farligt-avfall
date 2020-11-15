use std::{fmt, error, collections::HashMap};
use crate::pickup_event::PickUpEvent;
use rusoto_dynamodb::{DynamoDb, DynamoDbClient, WriteRequest, PutRequest, AttributeValue, BatchWriteItemInput, BatchWriteItemError, QueryInput};
use rusoto_core::{Region, RusotoError};

#[derive(fmt::Debug)]
pub struct EventsRepoError {
    pub errors: Vec<RusotoError<BatchWriteItemError>>,
}
impl fmt::Display for EventsRepoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for error in &self.errors {
            write!(f, "{}\n", error)?;
        }
        write!(f, "Total db errors while writing events: {}", self.errors.len())
    }
}
impl error::Error for EventsRepoError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None 
    }
}

pub async fn get_by_date(table: String, region: Region, date: String) -> Result<Vec<PickUpEvent>, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let client = DynamoDbClient::new(region);
    let mut attribute_values = HashMap::new();
    attribute_values.insert(":date".to_string(), AttributeValue{
        s: Some(date),
        ..Default::default()
    });
    let events: Vec<PickUpEvent> = client
        .query(QueryInput{
            table_name: table,
            expression_attribute_values: Some(attribute_values),
            key_condition_expression: Some("event_date = :date".to_string()),
            ..Default::default()
        })
        .await?
        .items
        .unwrap_or_else(|| vec![])
        .into_iter()
        .map(|item| {
            PickUpEvent::new(
                item.get("street").unwrap().s.as_ref().unwrap().clone(),
                item.get("district").unwrap().s.as_ref().unwrap().clone(),
                match item.get("description") {
                    Some(description) => Some(description.s.as_ref().unwrap().clone()),
                    None => None,
                },
                item.get("start_time").unwrap().s.as_ref().unwrap().clone(),
                item.get("end_time").unwrap().s.as_ref().unwrap().clone(),
            ).unwrap()
        })
        .collect();
    Ok(events)
}

pub async fn store(table: String, region: Region, events: Vec::<PickUpEvent>) -> Result<usize, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let client = DynamoDbClient::new(region);
    let write_requests: Vec<WriteRequest> = events.into_iter()
        .map(|event| {
            let mut attributes: HashMap<String, AttributeValue> = HashMap::new(); 
            attributes.insert("event_date".to_string(), AttributeValue{
                s: Some(event.date),
                ..Default::default()
            });
            attributes.insert("location_id".to_string(), AttributeValue{
                s: Some(event.location_id),
                ..Default::default()
            });
            attributes.insert("district".to_string(), AttributeValue{
                s: Some(event.district),
                ..Default::default()
            });
            attributes.insert("street".to_string(), AttributeValue{
                s: Some(event.street),
                ..Default::default()
            });
            match event.description.is_some() {
                true => {
                    attributes.insert("description".to_string(), AttributeValue{
                        s: Some(event.description.unwrap()),
                        ..Default::default()
                    });
                },
                _ => {}
            }
            attributes.insert("start_time".to_string(), AttributeValue{
                s: Some(event.time_start),
                ..Default::default()
            });
            attributes.insert("end_time".to_string(), AttributeValue{
                s: Some(event.time_end),
                ..Default::default()
            });
            WriteRequest{
                put_request: Some(PutRequest{
                    item: attributes
                }),
                ..Default::default()
            }
        })
        .collect();
    let mut batch_write_requests: Vec<BatchWriteItemInput> = Vec::new();
    for chunk in write_requests.chunks(25) {
        let chunk = chunk.to_vec();
        let mut request_items: HashMap<String, Vec<WriteRequest>> = HashMap::new();
        request_items.insert(table.clone(), chunk);
        batch_write_requests.push(BatchWriteItemInput{
            request_items,
            ..Default::default()
        });
    }
    let mut db_errors: Vec<RusotoError<BatchWriteItemError>> = Vec::new(); 
    let mut unprocessed_count = 0 as usize;
    for batch_write_request in batch_write_requests {
        match client.batch_write_item(batch_write_request).await {
            Ok(output) => match output.unprocessed_items {
                Some(unprocessed_items) => match unprocessed_items.get(&table) {
                    Some(items_for_table) => {
                        unprocessed_count += items_for_table.len(); // TODO: Do this in a less nested way..
                    },
                    None => {}
                },
                None => {}
            }
            Err(e) => {
                db_errors.push(e);
            },
        };
    }
    Ok(unprocessed_count)
}
