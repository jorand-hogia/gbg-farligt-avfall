use std::fmt;
use crate::models::PickUpEvent;
use rusoto_dynamodb::{DynamoDb, DynamoDbClient};
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

pub fn store(table: String, region: Region, events: Vec::<PickUpEvent>) -> Result<(), EventsRepoError> {
    let client = DynamoDbClient::new(region);
    events.into_iter()
        .map(|event| {
            println!("{}", event);
        });
    // Filter out elements in the past?
    // Filter out elements for long in the future? Six months? Since I use current year when parsing, and list is only updated twice a year
    // Store stuff to DynamoDb!
    Ok(())
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
        store("some-table".to_string(), region, vec![
            PickUpEvent::new("some-street".to_string(), "some-district".to_string(), None, "2020".to_string(), "2020".to_string())
        ]);
    }
}