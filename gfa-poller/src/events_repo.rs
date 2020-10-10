use std::fmt;
use crate::models::PickUpEvent;

#[derive(fmt::Debug)]
pub struct EventsRepoError {
    pub message: String
}
impl fmt::Display for EventsRepoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

pub fn store(table: String, events: Vec::<PickUpEvent>) -> Result<(), EventsRepoError> {
    for event in events {
        println!("{}", event);
    }
    // Filter out elements in the past?
    // Filter out elements for long in the future? Six months? Since I use current year when parsing, and list is only updated twice a year
    // Store stuff to DynamoDb!
    Ok(())
}