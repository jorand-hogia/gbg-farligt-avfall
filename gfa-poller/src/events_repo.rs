use std::fmt;
use std::collections::HashMap;
use crate::pickup_event::PickUpEvent;
use rusoto_dynamodb::{DynamoDb, DynamoDbClient, WriteRequest, PutRequest, AttributeValue, BatchWriteItemInput};
use rusoto_core::{Region};

#[derive(fmt::Debug)]
pub struct EventsRepoError {
    pub message: String
}
impl fmt::Display for EventsRepoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

pub async fn store(table: String, region: Region, events: Vec::<PickUpEvent>) -> Result<(), EventsRepoError> {
    let client = DynamoDbClient::new(region);
    let write_requests: Vec<WriteRequest> = events.into_iter()
        .map(|event| {
            let mut attributes: HashMap<String, AttributeValue> = HashMap::new(); 
            attributes.insert("event-date".to_string(), AttributeValue{
                s: Some(event.date()),
                ..Default::default()
            });
            attributes.insert("district".to_string(), AttributeValue{
                s: Some(event.district()),
                ..Default::default()
            });
            attributes.insert("street".to_string(), AttributeValue{
                s: Some(event.street()),
                ..Default::default()
            });
            match event.description().is_some() {
                true => {
                    attributes.insert("description".to_string(), AttributeValue{
                        s: Some(event.description().unwrap()),
                        ..Default::default()
                    });
                },
                _ => {}
            }
            attributes.insert("start_time".to_string(), AttributeValue{
                s: Some(event.start_time()),
                ..Default::default()
            });
            attributes.insert("end_time".to_string(), AttributeValue{
                s: Some(event.end_time()),
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
    let mut request_items: HashMap<String, Vec<WriteRequest>> = HashMap::new();
    request_items.insert(table, write_requests);
    let write_request = BatchWriteItemInput{
        request_items,
        ..Default::default()
    };
    match client.batch_write_item(write_request).await {
        Ok(_res) => return Ok(()), 
        Err(e) => return Err(EventsRepoError{
            message: format!("{}", e)
        })
    };
    // Filter out elements in the past?
    // Filter out elements for long in the future? Six months? Since I use current year when parsing, and list is only updated twice a year

    // https://docs.aws.amazon.com/AWSJavaSDK/latest/javadoc/com/amazonaws/services/dynamodbv2/model/PutItemRequest.html#PutItemRequest-java.lang.String-java.util.Map-
    // https://rusoto.github.io/rusoto/rusoto_dynamodb/struct.PutRequest.html
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temp() {
        let region = Region::Custom{
            name: "some-region".to_string(),
            endpoint: "http://testing.test".to_string()
        };
        let _res = store("some-table".to_string(), region, vec![
            PickUpEvent::new("some-street".to_string(), "some-district".to_string(), None, "2020-09-23T18:00:00+02:00".to_string(), "2020-09-23T18:45:00+02:00".to_string()).unwrap()
        ]);
    }
}
